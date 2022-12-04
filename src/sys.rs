use std::fmt;

use widestring::U16CStr;
use windows::core::*;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Memory::*;

#[derive(Debug)]
pub struct Win32Error {
    code: u32,
    msg: String,
}

impl fmt::Display for Win32Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (0x{:08x})", self.msg, self.code)
    }
}

impl std::error::Error for Win32Error {}

type Win32Result<T> = std::result::Result<T, Win32Error>;

macro_rules! win32 {
    ( $call:expr, $sentinel:ident ) => {
        {
            unsafe {
                let ret = $call;
                match ret {
                    $sentinel => {
                        let err = WSAGetLastError();
                        let mut buf: usize = 0;
                        let sz_buf = FormatMessageW(
                            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
                            None,
                            err.0 as u32,
                            0,
                            PWSTR::from_raw(&mut buf as *mut usize as *mut u16),
                            0,
                            None,
                        );
                        let buf_str = std::mem::transmute::<usize, *const u16>(buf);
                        let buf_str = U16CStr::from_ptr(buf_str, sz_buf as usize).unwrap();
                        let res = Err(Win32Error {
                            code: err.0 as u32,
                            msg: buf_str.to_string().unwrap(),
                        });
                        LocalFree(buf as isize);
                        res
                    }
                    _ => Ok(ret),
                }
            }
        }
    };
}

pub fn wsa_startup() -> Win32Result<i32> {
    let mut wsa_data: WSADATA;
    win32!(
        WSAStartup(2 << 16 | 2, &wsa_data as *mut WSAData)
    )
}

pub fn create_socket() -> Win32Result<SOCKET> {
    let res = win32!(
        WSASocketW(
            //AF_INET.0 as i32,
            12345,
            SOCK_RAW as i32,
            IPPROTO_ICMP.0,
            None,
            0,
            WSA_FLAG_OVERLAPPED
        ),
        INVALID_SOCKET
    );
    res
}

/*
fn create_socket2() -> Win32Result<SOCKET> {
    let res = {
        unsafe {
            let ret = WSASocketW(
                AF_INET.0 as i32,
                SOCK_RAW as i32,
                IPPROTO_ICMP.0,
                None,
                0,
                WSA_FLAG_OVERLAPPED,
            );
            match ret {
                INVALID_SOCKET => {
                    let err = WSAGetLastError();
                    let mut buf: usize = 0;
                    let sz_buf = FormatMessageW(
                        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
                        None,
                        err.0 as u32,
                        0,
                        PWSTR::from_raw(&mut buf as *mut usize as *mut u16),
                        0,
                        None,
                    );
                    let buf_str = std::mem::transmute::<usize, *const u16>(buf);
                    let buf_str = U16CStr::from_ptr(buf_str, sz_buf as usize).unwrap();
                    let res = Err(Win32Error {
                        code: err.0 as u32,
                        msg: buf_str.to_string().unwrap(),
                    });
                    LocalFree(buf as isize);
                    res
                }
                _ => Ok(ret),
            }
        }
    };
    res
}
*/
