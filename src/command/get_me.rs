use bytes::Bytes;

use super::{Code, Command};

const GET_ME: Code = 20;

#[derive(Debug)]
pub struct GetMe;

impl Command for GetMe {
    const CODE: Code = GET_ME;
    type ResponseModel = ();

    fn from_response(_: Bytes) -> Result<Self::ResponseModel, super::ResponseError> {
        todo!()
    }
}

impl From<GetMe> for Bytes {
    fn from(_: GetMe) -> Self {
        Bytes::new()
    }
}
