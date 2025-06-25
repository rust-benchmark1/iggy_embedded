mod get_me;
mod login_user;
mod ping;

use core::fmt::Debug;

pub use get_me::GetMe;
pub use login_user::LoginUser;
pub use ping::Ping;

use bytes::{BufMut as _, Bytes, BytesMut};

type Code = u32;

const LENGTH_BYTES: usize = 4;
const STATUS_BYTES: usize = 4;

#[derive(Debug)]
pub struct ResponseError;

pub trait Command: Debug + Into<Bytes> {
    const CODE: Code;
    type ResponseModel;

    fn code(&self) -> Code {
        Self::CODE
    }

    fn into_request(self) -> Bytes {
        let payload: Bytes = self.into();
        let total_length = LENGTH_BYTES + STATUS_BYTES + payload.len();

        let mut data = BytesMut::with_capacity(total_length);
        data.put_u32_le((STATUS_BYTES + payload.len()) as u32);
        data.put_u32_le(Self::CODE);
        data.put_slice(&payload);

        data.freeze()
    }

    fn from_response(response: Bytes) -> Result<Self::ResponseModel, ResponseError>;
}
