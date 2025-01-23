pub enum ResponseContent {
    Sized(Vec<u8>),
    // ZstdDecoderReader(zstd::stream::read::Decoder<>) ?
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
            ResponseContent::Stream(reader) => {
                use tokio_util::compat::FuturesAsyncReadCompatExt as _;
                resp.streamed_body(futures::io::AllowStdIo::new(reader).compat());
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
