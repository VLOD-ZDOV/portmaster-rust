//! Kernel TCP byte counter queries via `NETLINK_SOCK_DIAG`.
//!
//! Reads per-socket `tcp_info` (`tcpi_bytes_received`, `tcpi_bytes_acked`, and
//! `tcpi_bytes_sent`) directly over `AF_NETLINK` without needing writable
//! `/proc/sys/net/netfilter/nf_conntrack_acct`.

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::os::fd::OwnedFd;

use rustix::net::{
    bind, netlink::SocketAddrNetlink, recv, sendto, socket, AddressFamily, RecvFlags, SendFlags,
    SocketType,
};

use crate::conntrack::{attrs, check_error, messages, Family, NLMSG_DONE, NLMSG_ERROR, NLM_F_DUMP, NLM_F_REQUEST};

const SOCK_DIAG_BY_FAMILY: u16 = 20;
const IPPROTO_TCP: u8 = 6;
const INET_DIAG_INFO: u16 = 2;

const INET_DIAG_MSG_LEN: usize = 72;
const TCP_INFO_ACKED_OFF: usize = 120;
const TCP_INFO_RECEIVED_OFF: usize = 128;
const TCP_INFO_SENT_OFF: usize = 200;

/// Per-socket TCP byte counters from Linux `inet_diag_msg` + `tcp_info`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpCounter {
    pub local: IpAddr,
    pub local_port: u16,
    pub remote: IpAddr,
    pub remote_port: u16,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// Persistent `NETLINK_SOCK_DIAG` socket with a reusable receive buffer.
pub struct SockDiag {
    fd: OwnedFd,
    seq: u32,
    rx_buf: Vec<u8>,
}

impl SockDiag {
    /// Opens a `NETLINK_SOCK_DIAG` socket and binds it for kernel replies.
    pub fn open() -> io::Result<SockDiag> {
        let fd = socket(
            AddressFamily::NETLINK,
            SocketType::RAW,
            Some(rustix::net::netlink::SOCK_DIAG),
        )?;
        bind(&fd, &SocketAddrNetlink::new(0, 0))?;
        Ok(SockDiag {
            fd,
            seq: 0,
            rx_buf: vec![0u8; 1 << 16],
        })
    }

    /// Appends active TCP socket byte counters for `family` into `entries`.
    pub fn dump_tcp_counters_into(
        &mut self,
        family: Family,
        entries: &mut Vec<TcpCounter>,
    ) -> io::Result<()> {
        let req = build_tcp_diag_request(self.next_seq(), family);
        sendto(
            &self.fd,
            &req,
            SendFlags::empty(),
            &SocketAddrNetlink::new(0, 0),
        )?;

        if self.rx_buf.len() < (1 << 16) {
            self.rx_buf.resize(1 << 16, 0);
        }
        loop {
            let (n, _) = recv(&self.fd, &mut self.rx_buf[..], RecvFlags::empty())?;
            let mut done = false;
            for (kind, _seq, payload) in messages(&self.rx_buf[..n]) {
                match kind {
                    NLMSG_DONE => done = true,
                    NLMSG_ERROR => check_error(payload)?,
                    _ => {
                        if let Some(entry) = parse_tcp_diag_entry(payload) {
                            if entry.bytes_received > 0 || entry.bytes_sent > 0 {
                                entries.push(entry);
                            }
                        }
                    }
                }
            }
            if done {
                return Ok(());
            }
        }
    }

    fn next_seq(&mut self) -> u32 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }
}

fn build_tcp_diag_request(seq: u32, family: Family) -> [u8; 72] {
    // 16-byte nlmsghdr + 56-byte inet_diag_req_v2.
    let mut msg = [0u8; 72];
    msg[0..4].copy_from_slice(&(72u32).to_ne_bytes());
    msg[4..6].copy_from_slice(&SOCK_DIAG_BY_FAMILY.to_ne_bytes());
    msg[6..8].copy_from_slice(&(NLM_F_REQUEST | NLM_F_DUMP).to_ne_bytes());
    msg[8..12].copy_from_slice(&seq.to_ne_bytes());

    let req = &mut msg[16..];
    req[0] = family.af();
    req[1] = IPPROTO_TCP;
    req[2] = 1 << (INET_DIAG_INFO - 1);
    req[4..8].copy_from_slice(&u32::MAX.to_ne_bytes());
    msg
}

fn read_ne_u64(buf: &[u8], off: usize) -> Option<u64> {
    let bytes: [u8; 8] = buf.get(off..off + 8)?.try_into().ok()?;
    Some(u64::from_ne_bytes(bytes))
}

fn parse_tcp_diag_entry(payload: &[u8]) -> Option<TcpCounter> {
    if payload.len() < INET_DIAG_MSG_LEN {
        return None;
    }
    let af = payload[0];
    let local_port = u16::from_be_bytes([payload[4], payload[5]]);
    let remote_port = u16::from_be_bytes([payload[6], payload[7]]);
    if remote_port == 0 {
        return None;
    }
    let (local, remote) = match af {
        2 => {
            let src = <[u8; 4]>::try_from(payload.get(8..12)?).ok()?;
            let dst = <[u8; 4]>::try_from(payload.get(24..28)?).ok()?;
            (IpAddr::V4(Ipv4Addr::from(src)), IpAddr::V4(Ipv4Addr::from(dst)))
        }
        10 => {
            let src = <[u8; 16]>::try_from(payload.get(8..24)?).ok()?;
            let dst = <[u8; 16]>::try_from(payload.get(24..40)?).ok()?;
            (IpAddr::V6(Ipv6Addr::from(src)), IpAddr::V6(Ipv6Addr::from(dst)))
        }
        _ => return None,
    };

    let mut bytes_received = 0u64;
    let mut bytes_sent = 0u64;
    for (kind, value, _) in attrs(&payload[INET_DIAG_MSG_LEN..]) {
        if kind == INET_DIAG_INFO {
            let rx = read_ne_u64(value, TCP_INFO_RECEIVED_OFF).unwrap_or(0);
            let acked = read_ne_u64(value, TCP_INFO_ACKED_OFF).unwrap_or(0);
            let sent = read_ne_u64(value, TCP_INFO_SENT_OFF).unwrap_or(0);
            bytes_received = rx;
            bytes_sent = sent.max(acked);
            break;
        }
    }

    Some(TcpCounter {
        local,
        local_port,
        remote,
        remote_port,
        bytes_received,
        bytes_sent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_valid_tcp_diag_request() {
        let req = build_tcp_diag_request(7, Family::V4);
        assert_eq!(u32::from_ne_bytes(req[0..4].try_into().unwrap()), 72);
        assert_eq!(
            u16::from_ne_bytes(req[4..6].try_into().unwrap()),
            SOCK_DIAG_BY_FAMILY
        );
        assert_eq!(u32::from_ne_bytes(req[8..12].try_into().unwrap()), 7);
        assert_eq!(req[16], 2);
        assert_eq!(req[17], IPPROTO_TCP);
        assert_eq!(req[18], 2);
    }

    #[test]
    fn parses_ipv4_and_ipv6_tcp_info_counters() {
        let mut payload = vec![0u8; INET_DIAG_MSG_LEN];
        payload[0] = 2;
        payload[4..6].copy_from_slice(&54408u16.to_be_bytes());
        payload[6..8].copy_from_slice(&443u16.to_be_bytes());
        payload[8..12].copy_from_slice(&[192, 168, 0, 97]);
        payload[24..28].copy_from_slice(&[149, 154, 167, 51]);

        let mut tcp_info = vec![0u8; 208];
        tcp_info[TCP_INFO_ACKED_OFF..TCP_INFO_ACKED_OFF + 8].copy_from_slice(&3000u64.to_ne_bytes());
        tcp_info[TCP_INFO_RECEIVED_OFF..TCP_INFO_RECEIVED_OFF + 8]
            .copy_from_slice(&1976u64.to_ne_bytes());
        tcp_info[TCP_INFO_SENT_OFF..TCP_INFO_SENT_OFF + 8].copy_from_slice(&3049u64.to_ne_bytes());

        let nla_len = (4 + tcp_info.len()) as u16;
        payload.extend_from_slice(&nla_len.to_ne_bytes());
        payload.extend_from_slice(&INET_DIAG_INFO.to_ne_bytes());
        payload.extend_from_slice(&tcp_info);

        let entry = parse_tcp_diag_entry(&payload).unwrap();
        assert_eq!(entry.local, IpAddr::V4(Ipv4Addr::new(192, 168, 0, 97)));
        assert_eq!(entry.local_port, 54408);
        assert_eq!(entry.remote, IpAddr::V4(Ipv4Addr::new(149, 154, 167, 51)));
        assert_eq!(entry.remote_port, 443);
        assert_eq!(entry.bytes_received, 1976);
        assert_eq!(entry.bytes_sent, 3049);

        let mut payload6 = vec![0u8; INET_DIAG_MSG_LEN];
        payload6[0] = 10;
        payload6[4..6].copy_from_slice(&41000u16.to_be_bytes());
        payload6[6..8].copy_from_slice(&443u16.to_be_bytes());
        payload6[8..24].copy_from_slice(&Ipv6Addr::LOCALHOST.octets());
        payload6[24..40].copy_from_slice(&"2001:4860:4860::8888".parse::<Ipv6Addr>().unwrap().octets());
        payload6.extend_from_slice(&nla_len.to_ne_bytes());
        payload6.extend_from_slice(&INET_DIAG_INFO.to_ne_bytes());
        payload6.extend_from_slice(&tcp_info);

        let entry6 = parse_tcp_diag_entry(&payload6).unwrap();
        assert!(entry6.local.is_ipv6());
        assert_eq!(entry6.remote_port, 443);
        assert_eq!(entry6.bytes_received, 1976);
    }

    #[test]
    fn dumps_live_kernel_tcp_counters_without_root() {
        let Ok(mut diag) = SockDiag::open() else {
            return;
        };
        let mut out = Vec::new();
        diag.dump_tcp_counters_into(Family::V4, &mut out).unwrap();
        assert!(out.iter().all(|c| c.remote_port > 0 && (c.bytes_received > 0 || c.bytes_sent > 0)));
    }
}
