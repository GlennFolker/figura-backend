use std::sync::Arc;

use actix::{
    Actor,
    StreamHandler,
};
use actix_web::{
    web,
    HttpRequest,
    Responder,
};
use actix_web_actors::{
    ws,
    ws::{
        CloseReason,
        Message,
        ProtocolError,
        WebsocketContext,
    },
};
use uuid::Uuid;

use crate::{
    service::auth::AuthService,
    socket::message::{
        MsgError,
        WsCode,
        C2S,
        S2C,
    },
};

pub struct Socket {
    authorized: bool,
    auth: Arc<AuthService>,
}

impl Socket {
    pub fn start(auth: Arc<AuthService>, req: &HttpRequest, stream: web::Payload) -> impl Responder {
        ws::start(Self { authorized: false, auth }, req, stream)
    }
}

impl Actor for Socket {
    type Context = WebsocketContext<Self>;
}

impl StreamHandler<Result<Message, ProtocolError>> for Socket {
    fn handle(&mut self, item: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(Message::Binary(msg)) => match C2S::try_from(msg) {
                Ok(C2S::Token(token)) => {
                    let Ok(repr) = token.utf8_chunks().try_fold(String::new(), |mut repr, chunk| {
                        anyhow::ensure!(chunk.invalid().len() == 0);

                        repr.push_str(chunk.valid());
                        Ok(repr)
                    }) else {
                        ctx.close(Some(CloseReason {
                            code: WsCode::InvalidFramePayloadData.into(),
                            description: Some("invalid UTF-8 access token string".to_string()),
                        }));
                        return
                    };

                    let token = match Uuid::try_parse(&repr) {
                        Ok(token) => token,
                        Err(e) => {
                            ctx.close(Some(CloseReason {
                                code: WsCode::InvalidFramePayloadData.into(),
                                description: Some(format!("broken access token string: {e}")),
                            }));
                            return
                        }
                    };

                    if self.auth.check_access_token(token) {
                        self.authorized = true;
                        ctx.binary(S2C::Auth);
                    } else {
                        ctx.close(Some(CloseReason {
                            code: WsCode::Unauthorized.into(),
                            description: Some("invalid access token".to_string()),
                        }));
                    }
                }
                Ok(..) => {}
                Err(e) => ctx.close(Some(CloseReason {
                    code: match e {
                        MsgError::BadEnum(..) => WsCode::UnsupportedData,
                        MsgError::BadLength(..) => WsCode::InvalidFramePayloadData,
                    }
                    .into(),
                    description: Some(format!("{e}")),
                })),
            },
            Ok(..) => {}
            Err(..) => {}
        }
    }
}
