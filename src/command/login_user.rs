use bytes::{BufMut, Bytes, BytesMut};

use super::{Code, Command, ResponseError};

pub type UserId = u32;

const LOGIN_USER: Code = 38;

#[derive(Debug)]
pub struct LoginUser {
    username: &'static str,
    password: &'static str,
    version: Option<&'static str>,
    context: Option<&'static str>,
}

impl LoginUser {
    pub fn new(
        username: &'static str,
        password: &'static str,
        version: Option<&'static str>,
        context: Option<&'static str>,
    ) -> Self {
        Self {
            username,
            password,
            version,
            context,
        }
    }
}

#[derive(Debug)]
pub struct IdentityInfo {
    pub user_id: UserId,
    pub access_token: Option<u32>,
}

impl Command for LoginUser {
    const CODE: Code = LOGIN_USER;
    type ResponseModel = IdentityInfo;

    fn from_response(response: Bytes) -> Result<Self::ResponseModel, super::ResponseError> {
        let user_id = u32::from_le_bytes(response[..4].try_into().map_err(|_| ResponseError)?);

        Ok(Self::ResponseModel {
            user_id,
            access_token: None,
        })
    }
}

impl From<LoginUser> for Bytes {
    fn from(cmd: LoginUser) -> Self {
        let mut payload_len = 1 + cmd.username.len() + 1 + cmd.password.len();
        if let Some(version) = cmd.version {
            payload_len += 4 + version.len()
        }
        if let Some(context) = cmd.context {
            payload_len += 4 + context.len()
        }

        let mut bytes = BytesMut::with_capacity(payload_len);

        bytes.put_u8(cmd.username.len() as u8);
        bytes.put_slice(cmd.username.as_bytes());

        bytes.put_u8(cmd.password.len() as u8);
        bytes.put_slice(cmd.password.as_bytes());

        bytes.put_u32_le(cmd.version.map_or(0, |v| v.len() as u32));
        if let Some(version) = cmd.version {
            bytes.put_slice(version.as_bytes());
        }

        bytes.put_u32_le(cmd.context.map_or(0, |v| v.len() as u32));
        if let Some(context) = cmd.context {
            bytes.put_slice(context.as_bytes());
        }

        bytes.freeze()
    }
}
