// Commands from user space

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(Clone, Copy, FromPrimitive, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum CommandType {
    Shutdown              = 0,
    Verdict               = 1,
    UpdateV4              = 2,
    UpdateV6              = 3,
    ClearCache            = 4,
    GetLogs               = 5,
    GetBandwidthStats     = 6,
    PrintMemoryStats      = 7,
    CleanEndedConnections = 8,
}

#[repr(C, packed)]
pub struct Command {
    pub command_type: CommandType,
    value: [u8; 0],
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Eq)]
pub struct Verdict {
    pub id: u64,
    pub verdict: u8,
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Eq)]
pub struct UpdateV4 {
    pub protocol: u8,
    pub local_address: [u8; 4],
    pub local_port: u16,
    pub remote_address: [u8; 4],
    pub remote_port: u16,
    pub verdict: u8,
}

#[repr(C, packed)]
#[derive(Debug, PartialEq, Eq)]
pub struct UpdateV6 {
    pub protocol: u8,
    pub local_address: [u8; 16],
    pub local_port: u16,
    pub remote_address: [u8; 16],
    pub remote_port: u16,
    pub verdict: u8,
}

pub fn parse_type(bytes: &[u8]) -> Option<CommandType> {
    if bytes.is_empty() {
        return None;
    }
    FromPrimitive::from_u8(bytes[0])
}

pub fn parse_verdict(bytes: &[u8]) -> Option<&Verdict> {
    as_type(bytes)
}

pub fn parse_update_v4(bytes: &[u8]) -> Option<&UpdateV4> {
    as_type(bytes)
}

pub fn parse_update_v6(bytes: &[u8]) -> Option<&UpdateV6> {
    as_type(bytes)
}

fn as_type<T>(bytes: &[u8]) -> Option<&T> {
    if bytes.len() < core::mem::size_of::<T>() {
        return None;
    }
    let ptr: *const u8 = bytes.as_ptr();
    if (ptr as usize) % core::mem::align_of::<T>() != 0 {
        return None;
    }
    let t_ptr: *const T = ptr as _;
    unsafe { t_ptr.as_ref() }
}

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Read;
#[cfg(test)]
use std::mem::size_of;
#[cfg(test)]
use std::panic;

#[test]
fn test_go_command_file() {
    let mut file = File::open("testdata/go_command_test.bin").unwrap();
    loop {
        let mut command: [u8; 1] = [0];
        let bytes_count = file.read(&mut command).unwrap();
        if bytes_count == 0 {
            return;
        }
        if let Some(command) = parse_type(&command) {
            match command {
                CommandType::Shutdown => {}
                CommandType::Verdict => {
                    let mut buf = [0; size_of::<Verdict>()];
                    let bytes_count = file.read(&mut buf).unwrap();
                    if bytes_count != size_of::<Verdict>() {
                        panic!("unexpected bytes count")
                    }

                    assert_eq!(parse_verdict(&buf), Some(&Verdict { id: 1, verdict: 2 }))
                }
                CommandType::UpdateV4 => {
                    let mut buf = [0; size_of::<UpdateV4>()];
                    let bytes_count = file.read(&mut buf).unwrap();
                    if bytes_count != size_of::<UpdateV4>() {
                        panic!("unexpected bytes count")
                    }

                    assert_eq!(
                        parse_update_v4(&buf),
                        Some(&UpdateV4 {
                            protocol: 1,
                            local_address: [1, 2, 3, 4],
                            local_port: 2,
                            remote_address: [2, 3, 4, 5],
                            remote_port: 3,
                            verdict: 4
                        })
                    )
                }
                CommandType::UpdateV6 => {
                    let mut buf = [0; size_of::<UpdateV6>()];
                    let bytes_count = file.read(&mut buf).unwrap();
                    if bytes_count != size_of::<UpdateV6>() {
                        panic!("unexpected bytes count")
                    }

                    assert_eq!(
                        parse_update_v6(&buf),
                        Some(&UpdateV6 {
                            protocol: 1,
                            local_address: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                            local_port: 2,
                            remote_address: [
                                2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17
                            ],
                            remote_port: 3,
                            verdict: 4
                        })
                    )
                }
                CommandType::ClearCache => {}
                CommandType::GetLogs => {}
                CommandType::GetBandwidthStats => {}
                CommandType::PrintMemoryStats => {}
                CommandType::CleanEndedConnections => {}
            }
        } else {
            panic!("Unknown command: {}", command[0]);
        }
    }
}

#[test]
fn test_buffer_bounds_check() {
    let empty: [u8; 0] = [];
    assert_eq!(parse_type(&empty), None);
    assert_eq!(parse_verdict(&empty), None);
    assert_eq!(parse_update_v4(&empty), None);
    assert_eq!(parse_update_v6(&empty), None);

    let undersized = [1u8; 3];
    assert_eq!(parse_verdict(&undersized), None);
    assert_eq!(parse_update_v4(&undersized), None);
    assert_eq!(parse_update_v6(&undersized), None);
}
