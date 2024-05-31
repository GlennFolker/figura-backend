use std::str::FromStr;

use actix_web::{
    error::ParseError,
    http::header,
    HttpMessage,
};
use thiserror::Error;
use uuid::Uuid;

use crate::encode_uuid;

pub struct UserAgent {
    pub name: String,
    pub version: String,
}

#[derive(Error, Debug)]
#[error("invalid `user-agent` string: expected `{{name}}/{{version}}`")]
pub struct UserAgentParseError;
impl header::TryIntoHeaderValue for UserAgent {
    type Error = header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        let mut agent = self.name;
        agent.push_str(&self.version);

        header::HeaderValue::from_str(&agent)
    }
}

impl header::Header for UserAgent {
    fn name() -> header::HeaderName {
        header::USER_AGENT
    }

    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, ParseError> {
        header::from_one_raw_str(msg.headers().get(Self::name()))
    }
}

impl FromStr for UserAgent {
    type Err = UserAgentParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let slash = s.find('/').ok_or(UserAgentParseError)?;
        if slash == 0 || slash == s.len() - 1 {
            Err(UserAgentParseError)
        } else {
            Ok(Self {
                name: s[0..slash].to_string(),
                version: s[slash + 1..].to_string(),
            })
        }
    }
}

pub struct AccessToken(pub Uuid);
impl header::TryIntoHeaderValue for AccessToken {
    type Error = header::InvalidHeaderValue;

    fn try_into_value(self) -> Result<header::HeaderValue, Self::Error> {
        header::HeaderValue::from_str(&encode_uuid(self.0))
    }
}

impl header::Header for AccessToken {
    fn name() -> header::HeaderName {
        header::HeaderName::from_static("token")
    }

    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, ParseError> {
        header::from_one_raw_str(msg.headers().get(Self::name()))
    }
}

impl FromStr for AccessToken {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::try_parse(s)?))
    }
}
