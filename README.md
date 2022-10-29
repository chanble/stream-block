stream-block
===============================================

a test prject read stream with block type

这个项目源自[bollard](https://github.com/fussybeaver/bollard/issues/266)， 当我写用rust试图写一个docker客户端，写到了拉镜像(pull image)时， `bollard`提供的接口返回了一个`stream`。
当我写个单元测试的时候，用`tokio.block_on`转换成同步的时候发现不能拉成功，尝试了各种方法都不行。
如： 
```rust

    pub fn pull_image(
        &self,
        params: PullParams,
    ) -> impl Stream<Item = Result<PullResult, Error>> {
        let options = pull_param_to_create_image(params);
        self.client
            .create_image(Some(options), None, None)
            .map(|v| v.map(|vv| vv.into()))
    }

    #[test]
    fn test_pull_image() {
        let docker_res = Docker::connect_with_http(false, TEST_HOST, TEST_PORT);
        let docker = docker_res.unwrap();
        let mut stream = docker.pull_image(PullParams {
            from_image: "ibmcom/helloworld".to_owned(),
            ..Default::default()
        });
        while let Some(result) = tokio_test::block_on(stream.next()) {
            /// 这里只打印前三行
            /// ```
            /// running 1 test
            /// Ok(PullResult { id: Some("latest"), status: PullingFrom("ibmcom/helloworld"), progress: None, progress_detail: None })
            /// Ok(PullResult { id: Some("5843afab3874"), status: PullingFsLayer, progress: None, progress_detail: Some(PullResultProgressDetail { current: None, total: None }) })
            /// Ok(PullResult { id: Some("42cb94a98d49"), status: PullingFsLayer, progress: None, progress_detail: Some(PullResultProgressDetail { current: None, total: None }) })
            /// test docker::tests::test_pull_image ... ok
            /// ```
            println!("{:?}", result);
            assert!(result.is_ok())
        }
    }
```
而如果我自己模拟一个stream的话，则可以block成功。
如： 
```rust
use std::{thread, time::{Duration, SystemTime, UNIX_EPOCH}};

use futures_util::{Stream, StreamExt};

/// 
/// cargo.toml
/// 
/// ```toml
/// [package]
/// name = "stream-test"
/// version = "0.1.0"
/// edition = "2021"
/// 
/// # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
/// 
/// [dependencies]
/// tokio-stream = "0.1"
/// futures-util = "0.3"
/// tokio-test = "0.4"
/// ```
/// 
fn main() {
    let mut stream = test();
    while let Some(v) = tokio_test::block_on(stream.next()) {
        // 这里可以打印全部数据
        println!("time: {:?} v: {}", SystemTime::now().duration_since(UNIX_EPOCH), v);
    }
}

pub fn test() -> impl Stream<Item = i32> {
    StreamExt::map(tokio_stream::iter([1, 2, 3]), |v| {
        thread::sleep(Duration::from_secs(3));
        v
    })
}

```

于是，进行了一番探索，自己写了一个服务器和客户端。发现同步的时候只需要循环获取`response`即可。
如：

```rust
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

    let client = Client::new();
    let req_builder = Builder::new().method(Method::POST);
    let req = req_builder
        .uri("http://127.0.0.1:3000")
        .body(Body::empty()).unwrap();
    let mut response = rt.block_on(client.request(req)).unwrap();

    while let Some(data) = rt.block_on(response.data()) {
        // println!("aaaaaaaaaaaaaaaaaaaaaaa: {:?}", data);
        if let Ok(d) = data {
            println!("block {:?}: {:?}", Local::now(), d);
            // tx.send(string);
        } else {
            println!("data_stream next value {:?}", data);
        }
    }

```

然后，再仔细查看`bollard`源码， 发现它对response进行了封装。
如下：
```rust

    fn decode_into_stream<T>(res: Response<Body>) -> impl Stream<Item = Result<T, Error>>
    where
        T: DeserializeOwned,
    {
        FramedRead::new(
            StreamReader::new(res.into_body().map_err(Error::from)),
            JsonLineDecoder::new(),
        )
    }


// StreamReader
/// [StreamReader.src](https://github.com/fussybeaver/bollard/blob/master/src/read.rs#L179)
pin_project! {
    #[derive(Debug)]
    pub(crate) struct StreamReader<S> {
        #[pin]
        stream: S,
        state: ReadState,
    }
}

impl<S> tokio::io::AsyncRead for StreamReader<S>
where
    S: Stream<Item = Result<Bytes, Error>>,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        read_buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // ...
        return Poll::Ready(Ok(()));
    }
}

```

从代码中可以看到，response被装成了`tokio::io::AsyncRead`， 那么问题又来了， 封装成这个就不能进行`block_on`了？ 
于是继续看tokio的`block_on`，发现可疑之处。
代码如下：
```rust
loop {
    if let Some(core) = self.take_core() {
        return core.block_on(future);
    } else {
        let mut enter = crate::runtime::enter(false);

        let notified = self.notify.notified();
        pin!(notified);

        if let Some(out) = enter
            .block_on(poll_fn(|cx| {
                if notified.as_mut().poll(cx).is_ready() {
                    return Ready(None);
                }

                if let Ready(out) = future.as_mut().poll(cx) {
                    return Ready(Some(out));
                }

                Pending
            }))
            .expect("Failed to `Enter::block_on`")
        {
            return out;
        }
    }
}

```

猜测： 返现返回`Ready(T)`后，应该就退出了，而`AsyncRead`每次都会返回。导致没有接受完数据就退出了。

