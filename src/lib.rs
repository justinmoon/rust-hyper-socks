//! SOCKS proxy support for Hyper clients
#![doc(html_root_url = "https://docs.rs/hyper-socks/0.4.0")]
#![warn(missing_docs)]

extern crate hyper;
extern crate socks;

use hyper::net::{HttpStream, NetworkConnector};
use socks::{Socks4Stream, Socks5Stream};
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

/// A connector that will produce HttpStreams proxied over a SOCKS4 server.
#[derive(Debug)]
pub struct Socks4HttpConnector {
    addrs: Vec<SocketAddr>,
    userid: String,
}

impl Socks4HttpConnector {
    /// Creates a new `Socks4HttpConnector` which will connect to the specified
    /// proxy with the specified userid.
    pub fn new<T: ToSocketAddrs>(proxy: T, userid: &str) -> io::Result<Socks4HttpConnector> {
        Ok(Socks4HttpConnector {
            addrs: (proxy.to_socket_addrs())?.collect(),
            userid: userid.to_owned(),
        })
    }
}

impl NetworkConnector for Socks4HttpConnector {
    type Stream = HttpStream;

    fn connect(&self, host: &str, port: u16, scheme: &str) -> hyper::Result<HttpStream> {
        if scheme != "http" {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "invalid scheme for HTTP").into(),
            );
        }

        let socket = (Socks4Stream::connect(&self.addrs[..], (host, port), &self.userid))?;
        Ok(HttpStream(socket.into_inner()))
    }
}

/// A connector that will produce HttpStreams proxied over a SOCKS5 server.
#[derive(Debug)]
pub struct Socks5HttpConnector {
    addrs: Vec<SocketAddr>,
}

impl Socks5HttpConnector {
    /// Creates a new `Socks5HttpConnector` which will connect to the specified
    /// proxy.
    pub fn new<T: ToSocketAddrs>(proxy: T) -> io::Result<Socks5HttpConnector> {
        Ok(Socks5HttpConnector {
            addrs: proxy.to_socket_addrs()?.collect(),
        })
    }
}

impl NetworkConnector for Socks5HttpConnector {
    type Stream = HttpStream;

    fn connect(&self, host: &str, port: u16, scheme: &str) -> hyper::Result<HttpStream> {
        if scheme != "http" {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "invalid scheme for HTTP").into(),
            );
        }

        let socket = Socks5Stream::connect(&self.addrs[..], (host, port))?;
        Ok(HttpStream(socket.into_inner()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hyper;

    #[test]
    fn bitcoin_rpc() {
        // You need to provide these yourself in order to run this test
        let username = "";
        let password = "";
        let socks_host_and_port = "";
        let rpc_host_and_port = "";

        let connector = Socks5HttpConnector::new(socks_host_and_port).unwrap();
        let client = hyper::Client::with_connector(connector);

        let auth = hyper::header::Authorization(hyper::header::Basic {
            username: username.to_owned(),
            password: Some(password.to_owned()),
        });
        let response = client
            .post(rpc_host_and_port)
            .header(auth)
            .body(
                r#"{"jsonrpc": "1.0", "id":"curltest", "method": "getblockchaininfo", "params": []}"#,
                )
            .send()
            .unwrap();

        assert!(response.status.is_success());
    }
}
