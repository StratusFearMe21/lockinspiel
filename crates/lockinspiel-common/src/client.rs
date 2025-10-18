use std::num::ParseIntError;

use jiff::{SignedDuration, Timestamp};
use supabase_auth::models::{AuthClient, Session};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Request to server failed")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed to convert SystemTime to micros since Unix epoch")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("`current_clock_offset` couldn't split timestamps returned by server")]
    TimestampSplitError,
    #[error("Failed to parse an integer")]
    ParseError(#[from] ParseIntError),
    #[error("Failed to create timestamp from server times")]
    JiffError(#[from] jiff::Error),
    #[error("The offset was not present in the client for some reason")]
    NoOffsetCached,
}

pub struct LockinspielClient {
    offset: Option<SignedDuration>,
    session: Option<Session>,
    auth_client: AuthClient,
    offline: bool,
    client: reqwest::Client,
}

const BASE_URL: &'static str = dotenvy_macro::dotenv!("BASE_URL");
const PROJECT_URL: &'static str = dotenvy_macro::dotenv!("SUPABASE_URL");
const API_KEY: &'static str = dotenvy_macro::dotenv!("SUPABASE_API_KEY");
const JWT_SECRET: &'static str = dotenvy_macro::dotenv!("SUPABASE_JWT_SECRET");

impl Default for LockinspielClient {
    fn default() -> Self {
        Self {
            offset: None,
            session: None,
            auth_client: AuthClient::new(PROJECT_URL, API_KEY, JWT_SECRET),
            offline: false,
            client: reqwest::Client::new(),
        }
    }
}

impl LockinspielClient {
    /// The client is offline when the last attempt
    /// to contact the server failed. Abstractions
    /// like `now()` use this to prevent contacting
    /// the server unsuccesfully on every frame, making
    /// them immediate mode safe. Changing the offline
    /// status can be done by manually making successful
    /// contact with the server with `refresh_clock_offset()`
    /// or a function that calls it like `clock_offset()`
    #[inline]
    pub fn offline(&self) -> bool {
        self.offline
    }

    /// Uses the clock offset from the server
    /// to offset the current time
    pub async fn now(&mut self) -> jiff::Timestamp {
        let clock_offset = if self.offline {
            jiff::SignedDuration::from_secs(0)
        } else {
            match self.clock_offset().await {
                Ok(offset) => offset,
                Err(e) => {
                    tracing::error!(?e, "Failed to calculate clock offset");
                    jiff::SignedDuration::from_secs(0)
                }
            }
        };
        let now = jiff::Timestamp::now();
        now + clock_offset
    }

    /// Refreshes the client with the
    /// most accurate clock offset from
    /// the server.
    pub async fn refresh_clock_offset(&mut self) -> Result<(), ClientError> {
        self.offline = true;
        let request = self.client.get(format!("{}/time_sync", BASE_URL));
        let time1 = Timestamp::now();
        let response = request.send().await?;
        let time4 = Timestamp::now();
        let timestamps = response.text().await?;
        self.offline = false;
        let (time2, time3) = timestamps
            .split_once('\n')
            .ok_or(ClientError::TimestampSplitError)?;
        let time2 = Timestamp::from_microsecond(time2.parse::<i64>()?)?;
        let time3 = Timestamp::from_microsecond(time3.parse::<i64>()?)?;

        self.offset = Some((time2.duration_since(time1) + time3.duration_since(time4)) / 2);

        tracing::info!(?self.offset, "New offset");

        Ok(())
    }

    /// Gets the cached clock offset or refreshes it
    /// if there is not an offset cached
    pub async fn clock_offset(&mut self) -> Result<SignedDuration, ClientError> {
        if self.offset.is_none() {
            self.refresh_clock_offset().await?;
        }

        self.offset.ok_or(ClientError::NoOffsetCached)
    }

    /// Gets the cached clock offset if present
    pub fn cached_clock_offset(&self) -> Option<SignedDuration> {
        self.offset
    }
}
