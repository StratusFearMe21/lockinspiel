use std::{
    task::Poll,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    body::{Bytes, HttpBody},
    http::StatusCode,
};
use color_eyre::eyre::Context;
use http_body::Frame;
use pin_project::pin_project;
use tracing::instrument;

use crate::error::{self, WithStatusCode};

#[pin_project]
pub struct TimeDataStream(Duration, bool);

impl TimeDataStream {
    pub fn new(time: Duration) -> Self {
        Self(time, false)
    }
}

impl HttpBody for TimeDataStream {
    type Data = Bytes;
    type Error = error::Error;

    #[instrument(skip_all)]
    fn poll_frame(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        use std::fmt::Write;

        let this = self.project();
        if *this.1 {
            return Poll::Ready(None);
        } else {
            *this.1 = true;
        }
        let mut result = String::new();
        writeln!(result, "{}", this.0.as_micros()).unwrap();
        result.reserve(result.len() + 1);
        match SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .wrap_err("Failed to convert SystemTime to micros since Unix epoch")
        {
            Ok(duration) => write!(result, "{}", duration.as_micros()).unwrap(),
            Err(e) => {
                return Poll::Ready(Some(
                    Err(e).with_status_code(StatusCode::INTERNAL_SERVER_ERROR),
                ));
            }
        }

        Poll::Ready(Some(Ok(Frame::data(Bytes::from(result)))))
    }

    fn is_end_stream(&self) -> bool {
        self.1
    }

    #[instrument(skip_all)]
    fn size_hint(&self) -> http_body::SizeHint {
        let num_digits = self.0.as_micros().ilog10() as u64 + 1;
        http_body::SizeHint::with_exact((num_digits + 1) + num_digits)
    }
}
