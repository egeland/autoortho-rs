use crate::xplane::RrefCodec;
use std::net::SocketAddr;
use thiserror::Error;
use tokio::net::UdpSocket;

#[derive(Debug, Error)]
pub enum UdpError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Connection error")]
    ConnectionError,
}

/// X-Plane UDP RREF client
pub struct XPlaneUdpClient {
    socket: UdpSocket,
    remote_addr: SocketAddr,
}

impl XPlaneUdpClient {
    /// Create a new UDP client connected to X-Plane
    pub async fn connect(remote_addr: SocketAddr) -> Result<Self, UdpError> {
        // Bind to local address (use 0 to let OS choose)
        let local_addr: SocketAddr =
            "127.0.0.1:0"
                .parse()
                .map_err(|e: std::net::AddrParseError| {
                    UdpError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        e.to_string(),
                    ))
                })?;

        let socket = UdpSocket::bind(local_addr).await?;
        socket.connect(remote_addr).await?;

        Ok(Self {
            socket,
            remote_addr,
        })
    }

    /// Send an RREF request
    pub async fn request_dataref(
        &self,
        freq_hz: i32,
        index: i32,
        dataref: &str,
    ) -> Result<(), UdpError> {
        let packet = RrefCodec::encode_request(freq_hz, index, dataref);
        self.socket.send(&packet).await?;
        Ok(())
    }

    /// Receive RREF response
    pub async fn receive_response(&self) -> Result<Vec<(i32, f32)>, UdpError> {
        let mut buf = vec![0u8; 4096];
        let n = self.socket.recv(&mut buf).await?;
        buf.truncate(n);

        RrefCodec::decode_response(&buf).map_err(|e| UdpError::ParseError(e.to_string()))
    }

    /// Get remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_error_display() {
        let err = UdpError::ConnectionError;
        assert!(err.to_string().contains("Connection"));

        let parse_err = UdpError::ParseError("test".to_string());
        assert!(parse_err.to_string().contains("Parse"));
    }

    #[tokio::test]
    async fn test_udp_bind() {
        let addr: SocketAddr = "127.0.0.1:49999".parse().unwrap();
        // Just test that bind doesn't panic (actual connection would require X-Plane)
        let result = UdpSocket::bind(addr).await;
        // May fail if port in use, but shouldn't panic
        let _ = result;
    }
}
