// use futures::AsyncWriteExt;
use tokio_util::bytes::BufMut;

pub enum ResponseContent {
    Sized(Vec<u8>),
    // ZstdDecoderReader(zstd::stream::read::Decoder<>)
    Stream(Box<dyn std::io::Read + Send>),
}

impl From<Vec<u8>> for ResponseContent {
    fn from(value: Vec<u8>) -> Self {
        Self::Sized(value)
    }
}
impl From<String> for ResponseContent {
    fn from(value: String) -> Self {
        Self::Sized(value.into())
    }
}
impl From<&'static str> for ResponseContent {
    fn from(value: &'static str) -> Self {
        Self::Sized(value.as_bytes().to_vec())
    }
}

impl From<Box<dyn std::io::Read + Send>> for ResponseContent {
    fn from(value: Box<dyn std::io::Read + Send>) -> Self {
        Self::Stream(value)
    }
}

pub struct Response {
    status: rocket::http::Status,
    headers: std::collections::HashMap<String, String>,
    content: ResponseContent,
    content_type: rocket::http::ContentType,
}
pin_project_lite::pin_project! {

    #[derive(Debug)]
    struct CustomByteStream<T> {
        inner: T,
        total_read: usize
    }
}

impl<T: std::io::Read + Send> rocket::tokio::io::AsyncRead for CustomByteStream<T> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut rocket::tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        use std::io::Write as _;
        use std::task::Poll;
        let me = self.project();

        let mut buffer = [0; 1_000];

        match me.inner.read(&mut buffer) {
            Ok(read) => {
                *me.total_read += read;
                info!("Read {} bytes from inner stream", me.total_read);
                let wrote = buf.writer().write(&buffer[..read]).unwrap();
                if wrote == 0 {
                    error!("EOF");
                }

                Poll::Ready(Ok(()))
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl Response {
    pub fn status(&self) -> &rocket::http::Status {
        &self.status
    }

    pub fn headers(&self) -> &std::collections::HashMap<String, String> {
        &self.headers
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for Response {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        use {
            rocket::response::{Builder, Response},
            std::io::Cursor,
        };

        let mut resp = Builder::new(Response::default());

        resp.status(self.status);

        resp.raw_header("Content-Type", self.content_type.to_string());

        for (name, value) in self.headers.into_iter() {
            resp.raw_header(name, value);
        }

        match self.content {
            ResponseContent::Sized(vec) => {
                resp.sized_body(vec.len(), Cursor::new(vec));
            }
            ResponseContent::Stream(mut reader) => {
                use std::io::Write as _;
                use tokio_util::compat::FuturesAsyncReadCompatExt as _;
                // resp.streamed_body(CustomByteStream {
                //     inner: async_read,
                //     total_read: 0,
                // });
                // let mut total = Vec::new();

                // let mut buffer = [0; 10_000];
                // let mut total_read = 0;
                // let mut total_write = 0;

                // loop {
                //     let read = reader.read(&mut buffer).unwrap();

                //     if read == 0 {
                //         warn!("EOF");
                //         std::thread::sleep_ms(1000);
                //         assert_eq!(reader.read(&mut buffer).unwrap(), 0);
                //         break;
                //     }
                //     total_read += read;

                //     total_write += total.write(&buffer[..read]).unwrap();
                // }

                // debug!("Finished decoding");
                // println!("total_read: {total_read}\ntotal_write: {total_write}");
                // resp.sized_body(total.len(), Cursor::new(total));
                // let allow = futures::io::AllowStdIo::new(async_read);

                // let compat = allow.compat();

                // let stream = ReaderStream::from(custom);

                resp.streamed_body(futures::io::AllowStdIo::new(reader).compat());
                // let stream = async_stream::try_stream! {

                //     let data: Vec::<u8> = vec![0; 10];
                //     for i in 0..10{
                //         yield data.clone()
                //     }

                // };
                // StreamReader::new(futures::io::AllowStdIo::new(async_read).compat());

                // resp.streamed_body(stream);
            }
        }

        resp.ok()
    }
}

pub struct ResponseBuilder {
    inner: Response,
}

impl ResponseBuilder {
    // pub fn from_response(response: Response) -> Self {
    //     Self { inner: response }
    // }

    pub fn with_content(mut self, value: impl Into<ResponseContent>) -> Self {
        self.inner.content = value.into();
        self
    }

    pub fn with_content_type(
        mut self,
        ctype /*C TYPE badeu :D*/: rocket::http::ContentType,
    ) -> Self {
        self.inner.content_type = ctype;
        self
    }

    pub fn with_status(mut self, status: rocket::http::Status) -> Self {
        self.inner.status = status;
        self
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.inner
            .headers
            .insert(name.to_string(), value.to_string());
        self
    }

    pub fn build(self) -> Response {
        self.inner
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        use {
            rocket::http::{ContentType, Status},
            std::collections::HashMap,
        };

        ResponseBuilder {
            inner: Response {
                status: Status::Ok,
                headers: HashMap::new(),
                content: ResponseContent::Sized(Vec::new()),
                content_type: ContentType::Any,
            },
        }
    }
}
