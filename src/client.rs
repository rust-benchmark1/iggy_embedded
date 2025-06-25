use bytes::{BufMut as _, Bytes, BytesMut};
use embassy_net::tcp::{ConnectError, TcpSocket};
use embassy_net::IpEndpoint;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use log::{debug, error};

use crate::command::Command;

const RESPONSE_INITIAL_BYTES: usize = 8;

#[derive(Debug)]
pub struct ClientError;

// TODO: add state and other attrs
pub struct TcpClient<'a> {
    socket: Mutex<CriticalSectionRawMutex, TcpSocket<'a>>,
}

impl<'a> TcpClient<'a> {
    pub fn new(socket: TcpSocket<'a>) -> Self {
        let socket = Mutex::new(socket);

        Self { socket }
    }

    pub async fn connect<T: Into<IpEndpoint>>(
        &self,
        remote_endpoint: T,
    ) -> Result<(), ConnectError> {
        let mut socket = self.socket.lock().await;
        socket.connect(remote_endpoint).await
    }

    pub async fn send_command<T: Command>(&self, cmd: T) -> Result<Bytes, ClientError> {
        debug!("Sending command with code {:?}", cmd.code());
        let data = cmd.into_request();

        debug!("Trying to acquire mutex lock");
        let mut socket = self.socket.lock().await;
        debug!("Mutex lock acquired");

        debug!("Writing command to socket");
        socket.write(&data).await.unwrap();
        socket.flush().await.unwrap();

        debug!("Waiting for server response");
        let mut buf = [0u8; RESPONSE_INITIAL_BYTES];
        let bytes_read = socket.read(&mut buf).await.unwrap();
        if bytes_read != RESPONSE_INITIAL_BYTES {
            error!(
                "Expected to read {} bytes of response header but only received {}",
                RESPONSE_INITIAL_BYTES, bytes_read
            );
            return Err(ClientError);
        }

        let status = u32::from_le_bytes(buf[..4].try_into().map_err(|_| ClientError)?);
        if status != 0 {
            error!("Invalid return status: {}", status);
            return Err(ClientError);
        }

        let len = u32::from_le_bytes(buf[4..].try_into().map_err(|_| ClientError)?);
        debug!("Response payload has {} bytes", len);

        let mut buf = BytesMut::with_capacity(len as usize);
        buf.put_bytes(0, len as usize);

        debug!("Reading payload");
        let bytes_read = socket.read(&mut buf).await.map_err(|_| ClientError)?;
        if bytes_read != len as usize {
            error!(
                "Expected to read {} bytes but only received {}",
                len, bytes_read
            );
            return Err(ClientError);
        }

        Ok(buf.freeze())
    }
}
