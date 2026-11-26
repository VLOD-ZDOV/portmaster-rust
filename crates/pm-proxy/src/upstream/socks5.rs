use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const SOCKS5_VERSION: u8 = 0x05;
const SOCKS5_AUTH_NONE: u8 = 0x00;
const SOCKS5_AUTH_USER_PASS: u8 = 0x02;
const SOCKS5_AUTH_NO_ACCEPTABLE: u8 = 0xFF;
const SOCKS5_USER_PASS_VERSION: u8 = 0x01;
const SOCKS5_CMD_CONNECT: u8 = 0x01;
const SOCKS5_CMD_UDP_ASSOCIATE: u8 = 0x03;
const SOCKS5_ADDR_IPV4: u8 = 0x01;
const SOCKS5_ADDR_DOMAIN: u8 = 0x03;
const SOCKS5_ADDR_IPV6: u8 = 0x04;

async fn socks5_negotiate(
    stream: &mut TcpStream,
    username: Option<&str>,
    password: Option<&str>,
) -> io::Result<()> {
    if let (Some(user), pass) = (username, password) {
        // Offer both User/Pass and None
        stream
            .write_all(&[SOCKS5_VERSION, 2, SOCKS5_AUTH_NONE, SOCKS5_AUTH_USER_PASS])
            .await?;
        let mut resp = [0u8; 2];
        stream.read_exact(&mut resp).await?;
        if resp[0] != SOCKS5_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid socks5 version in auth response",
            ));
        }
        if resp[1] == SOCKS5_AUTH_NO_ACCEPTABLE {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "socks5: no acceptable authentication methods",
            ));
        }
        if resp[1] == SOCKS5_AUTH_USER_PASS {
            let user_bytes = user.as_bytes();
            let pass_bytes = pass.unwrap_or("").as_bytes();
            if user_bytes.len() > 255 || pass_bytes.len() > 255 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "socks5: credentials too long",
                ));
            }
            let mut auth_req = Vec::with_capacity(3 + user_bytes.len() + pass_bytes.len());
            auth_req.push(SOCKS5_USER_PASS_VERSION);
            auth_req.push(user_bytes.len() as u8);
            auth_req.extend_from_slice(user_bytes);
            auth_req.push(pass_bytes.len() as u8);
            auth_req.extend_from_slice(pass_bytes);

            stream.write_all(&auth_req).await?;
            let mut auth_resp = [0u8; 2];
            stream.read_exact(&mut auth_resp).await?;
            if auth_resp[1] != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "socks5: authentication failed",
                ));
            }
        }
    } else {
        stream
            .write_all(&[SOCKS5_VERSION, 1, SOCKS5_AUTH_NONE])
            .await?;
        let mut resp = [0u8; 2];
        stream.read_exact(&mut resp).await?;
        if resp[0] != SOCKS5_VERSION || resp[1] != SOCKS5_AUTH_NONE {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "socks5: server requires unsupported authentication",
            ));
        }
    }
    Ok(())
}

fn push_socks5_addr(buf: &mut Vec<u8>, host: &str, port: u16) -> io::Result<()> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(ipv4) => {
                buf.push(SOCKS5_ADDR_IPV4);
                buf.extend_from_slice(&ipv4.octets());
            }
            IpAddr::V6(ipv6) => {
                buf.push(SOCKS5_ADDR_IPV6);
                buf.extend_from_slice(&ipv6.octets());
            }
        }
    } else {
        let host_bytes = host.as_bytes();
        if host_bytes.is_empty() || host_bytes.len() > 255 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "socks5: host name empty or too long",
            ));
        }
        buf.push(SOCKS5_ADDR_DOMAIN);
        buf.push(host_bytes.len() as u8);
        buf.extend_from_slice(host_bytes);
    }
    buf.extend_from_slice(&port.to_be_bytes());
    Ok(())
}

fn reply_error_message(code: u8) -> String {
    match code {
        0x01 => "general SOCKS server failure".to_string(),
        0x02 => "connection not allowed by ruleset".to_string(),
        0x03 => "network unreachable".to_string(),
        0x04 => "host unreachable".to_string(),
        0x05 => "connection refused".to_string(),
        0x06 => "TTL expired".to_string(),
        0x07 => "command not supported".to_string(),
        0x08 => "address type not supported".to_string(),
        other => format!("socks5 error code 0x{:02x}", other),
    }
}

async fn read_socks5_reply(stream: &mut TcpStream) -> io::Result<(Option<IpAddr>, u16)> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    if header[0] != SOCKS5_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "socks5: invalid reply version",
        ));
    }
    if header[1] != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            reply_error_message(header[1]),
        ));
    }

    match header[3] {
        SOCKS5_ADDR_IPV4 => {
            let mut raw = [0u8; 6];
            stream.read_exact(&mut raw).await?;
            let ip = IpAddr::V4(Ipv4Addr::new(raw[0], raw[1], raw[2], raw[3]));
            let port = u16::from_be_bytes([raw[4], raw[5]]);
            Ok((Some(ip), port))
        }
        SOCKS5_ADDR_IPV6 => {
            let mut raw = [0u8; 18];
            stream.read_exact(&mut raw).await?;
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&raw[..16]);
            let ip = IpAddr::V6(Ipv6Addr::from(octets));
            let port = u16::from_be_bytes([raw[16], raw[17]]);
            Ok((Some(ip), port))
        }
        SOCKS5_ADDR_DOMAIN => {
            let len = stream.read_u8().await?;
            let mut buf = vec![0u8; len as usize + 2];
            stream.read_exact(&mut buf).await?;
            let host = std::str::from_utf8(&buf[..len as usize]).ok();
            let ip = host.and_then(|s| s.parse::<IpAddr>().ok());
            let port = u16::from_be_bytes([buf[len as usize], buf[len as usize + 1]]);
            Ok((ip, port))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "socks5: invalid address type in reply",
        )),
    }
}

pub async fn socks5_connect(
    stream: &mut TcpStream,
    username: Option<&str>,
    password: Option<&str>,
    host: &str,
    port: u16,
) -> io::Result<()> {
    socks5_negotiate(stream, username, password).await?;

    let mut req = Vec::with_capacity(32);
    req.extend_from_slice(&[SOCKS5_VERSION, SOCKS5_CMD_CONNECT, 0x00]);
    push_socks5_addr(&mut req, host, port)?;
    stream.write_all(&req).await?;

    let _ = read_socks5_reply(stream).await?;
    Ok(())
}

/// Executes a SOCKS5 UDP ASSOCIATE handshake and returns the relay `SocketAddr`.
/// The caller must keep `stream` alive for the duration of the UDP association.
pub async fn socks5_udp_associate(
    stream: &mut TcpStream,
    username: Option<&str>,
    password: Option<&str>,
    proxy_ip: IpAddr,
) -> io::Result<SocketAddr> {
    socks5_negotiate(stream, username, password).await?;

    let mut req = Vec::with_capacity(24);
    req.extend_from_slice(&[SOCKS5_VERSION, SOCKS5_CMD_UDP_ASSOCIATE, 0x00]);
    match proxy_ip {
        IpAddr::V4(_) => {
            req.push(SOCKS5_ADDR_IPV4);
            req.extend_from_slice(&[0; 4]);
        }
        IpAddr::V6(_) => {
            req.push(SOCKS5_ADDR_IPV6);
            req.extend_from_slice(&[0; 16]);
        }
    }
    req.extend_from_slice(&0u16.to_be_bytes());
    stream.write_all(&req).await?;

    let (bound_ip, bound_port) = read_socks5_reply(stream).await?;
    if bound_port == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "socks5: UDP ASSOCIATE returned port 0",
        ));
    }
    let relay_ip = match bound_ip {
        Some(ip) if !ip.is_unspecified() => ip,
        _ => proxy_ip,
    };
    Ok(SocketAddr::new(relay_ip, bound_port))
}

/// Encodes the RFC 1928 SOCKS5 UDP request header `[RSV=0,0, FRAG=0, ATYP, DST, PORT]`.
pub fn encode_udp_header(host: &str, port: u16) -> io::Result<Vec<u8>> {
    let mut header = Vec::with_capacity(24);
    header.extend_from_slice(&[0x00, 0x00, 0x00]);
    push_socks5_addr(&mut header, host, port)?;
    Ok(header)
}

/// Strips the RFC 1928 SOCKS5 UDP header and returns the application payload.
pub fn decode_udp_packet(packet: &[u8]) -> io::Result<&[u8]> {
    if packet.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "socks5 udp packet shorter than header",
        ));
    }
    if packet[0] != 0x00 || packet[1] != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "socks5 udp packet invalid RSV",
        ));
    }
    if packet[2] != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "socks5 udp fragmented packets are not supported",
        ));
    }
    let header_len = match packet[3] {
        SOCKS5_ADDR_IPV4 => 4 + 4 + 2,
        SOCKS5_ADDR_IPV6 => 4 + 16 + 2,
        SOCKS5_ADDR_DOMAIN => {
            if packet.len() < 5 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "socks5 udp domain packet truncated",
                ));
            }
            4 + 1 + packet[4] as usize + 2
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "socks5 udp invalid address type",
            ));
        }
    };
    if packet.len() < header_len {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "socks5 udp packet truncated inside address",
        ));
    }
    Ok(&packet[header_len..])
}
