use bytes::Bytes;

use super::{Code, Command};

const PING_CODE: Code = 1;

#[derive(Debug)]
pub struct Ping;

impl Command for Ping {
    const CODE: Code = PING_CODE;
    type ResponseModel = ();

    fn from_response(_: Bytes) -> Result<Self::ResponseModel, super::ResponseError> {
        Ok(())
    }
}

impl From<Ping> for Bytes {
    fn from(_: Ping) -> Self {
        Bytes::new()
    }
}
