pub struct Response {
    status: rocket::http::Status,
    headers: std::collections::HashMap<String, String>,
    content: Vec<u8>,
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

        for (name, value) in self.headers {
            resp.raw_header(name, value);
        }

        resp.sized_body(self.content.len(), Cursor::new(self.content));

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

    pub fn with_content(mut self, value: impl Into<Vec<u8>>) -> Self {
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
                content: Vec::new(),
                content_type: ContentType::Any,
            },
        }
    }
}

// pub struct JsonApiResponse {
//     json: JsonValue,
//     status: Status,
//     headers: std::collections::HashMap<String, String>,
// }

// impl<'r> rocket::response::Responder<'r, 'static> for JsonApiResponse {
//     fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'static> {
//         let mut resp = rocket::Response::build_from(self.json.respond_to(req).unwrap());

//         let mut resp = resp.status(self.status);

//         for (name, value) in self.headers {
//             resp = resp.raw_header(name, value);
//         }

//         let out = resp.ok();
//         // trace!("{out:?}");

//         out
//     }
// }

// pub struct JsonApiResponseBuilder {
//     inner: JsonApiResponse,
// }

// impl JsonApiResponseBuilder {
//     pub fn with_json(mut self, value: JsonValue) -> Self {
//         self.inner.json = value;
//         self
//     }

//     pub fn with_status(mut self, status: Status) -> Self {
//         self.inner.status = status;
//         self
//     }

//     pub fn with_header(mut self, name: &str, value: &str) -> Self {
//         self.inner
//             .headers
//             .insert(name.to_string(), value.to_string());
//         self
//     }

//     pub fn build(self) -> JsonApiResponse {
//         self.inner
//     }
// }

// impl Default for JsonApiResponseBuilder {
//     fn default() -> Self {
//         JsonApiResponseBuilder {
//             inner: JsonApiResponse {
//                 json: json!({}),
//                 status: Status::Ok,
//                 headers: {
//                     let mut h = std::collections::HashMap::new();
//                     h.insert("Content-Type".to_string(), "application/json".to_string());
//                     h
//                 },
//             },
//         }
//     }
// }
