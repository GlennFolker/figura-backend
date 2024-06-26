use std::ops::RangeInclusive;

use actix_web::web;
use actix_web_actors::ws::CloseCode;
use thiserror::Error;

#[repr(u16)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum WsCode {
    NormalClosure = 1000,
    GoingAway,
    ProtocolError,
    UnsupportedData,
    NoStatusReceived = 1005,
    AbnormalClosure,
    InvalidFramePayloadData,
    PolicyViolation,
    MessageTooBig,
    MandatoryExt,
    InternalError,
    ServiceRestart,
    TryAgainLater,
    BadGateway,
    TlsHandshake,
    Unauthorized = 3000,
    ReAuth = 4000,
    Banned,
    TooManyConnections,
}

impl From<WsCode> for CloseCode {
    fn from(value: WsCode) -> Self {
        match value {
            WsCode::NormalClosure => Self::Normal,
            WsCode::GoingAway => Self::Away,
            WsCode::ProtocolError => Self::Protocol,
            WsCode::UnsupportedData => Self::Unsupported,
            WsCode::NoStatusReceived => Self::Other(value as u16),
            WsCode::AbnormalClosure => Self::Abnormal,
            WsCode::InvalidFramePayloadData => Self::Invalid,
            WsCode::PolicyViolation => Self::Policy,
            WsCode::MessageTooBig => Self::Size,
            WsCode::MandatoryExt => Self::Extension,
            WsCode::InternalError => Self::Error,
            WsCode::ServiceRestart => Self::Restart,
            WsCode::TryAgainLater => Self::Again,
            WsCode::BadGateway => Self::Other(value as u16),
            WsCode::TlsHandshake => Self::Tls,
            WsCode::Unauthorized => Self::Other(value as u16),
            WsCode::ReAuth => Self::Other(value as u16),
            WsCode::Banned => Self::Other(value as u16),
            WsCode::TooManyConnections => Self::Other(value as u16),
        }
    }
}

#[derive(Error, Debug)]
pub enum MsgError {
    #[error("invalid value of {}: must be {} to {} inclusive, got {}", .0, .1.start(), .1.end(), .2)]
    BadEnum(&'static str, RangeInclusive<usize>, usize),
    #[error("invalid buffer size for {}: must be {} {} bytes, got {}", .0, if *.2 { "exactly" } else { "at least" }, .1, .3)]
    BadLength(&'static str, usize, bool, usize),
}

pub enum C2S {
    Token(web::Bytes),
    Ping(u32, bool, web::Bytes),
    Sub(u128),
    UnSub(u128),
}

impl TryFrom<web::Bytes> for C2S {
    type Error = MsgError;

    fn try_from(mut buf: web::Bytes) -> Result<Self, Self::Error> {
        if buf.len() == 0 {
            Err(MsgError::BadLength("C2S", 1, false, 0))
        } else {
            match buf[0] {
                0 => Ok(C2S::Token(buf.split_off(1))),
                1 => {
                    if buf.len() >= 6 {
                        Ok(C2S::Ping(
                            u32::from_be_bytes((&buf[1..5]).try_into().unwrap()),
                            buf[5] != 0,
                            buf.split_off(6),
                        ))
                    } else {
                        Err(MsgError::BadLength("C2S::Ping", 6, false, buf.len()))
                    }
                }
                2 => {
                    if buf.len() == 17 {
                        Ok(C2S::Sub(u128::from_be_bytes((&buf[1..]).try_into().unwrap())))
                    } else {
                        Err(MsgError::BadLength("C2S::Sub", 17, true, buf.len()))
                    }
                }
                3 => {
                    if buf.len() == 17 {
                        Ok(C2S::UnSub(u128::from_be_bytes((&buf[1..]).try_into().unwrap())))
                    } else {
                        Err(MsgError::BadLength("C2S::UnSub", 17, true, buf.len()))
                    }
                }
                other => Err(MsgError::BadEnum("C2S", 0..=3, other.into())),
            }
        }
    }
}

pub enum S2C {
    Auth,
}

impl Into<web::Bytes> for S2C {
    fn into(self) -> web::Bytes {
        match self {
            S2C::Auth => web::Bytes::from_static(&[0]),
        }
    }
}
