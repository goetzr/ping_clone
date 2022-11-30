use std::fmt;

use widestring::U16CStr;
use windows::core::*;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Memory::*;

#[derive(Debug)]
struct Win32Error {
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
    ( $call:expr, $sentinel:expr ) => {
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
                _ => Ok(ret)
            }
        }
    };
}

macro_rules! test {
    ( $call:expr, $sentinel:expr ) => {
        let ret = $call;
        match ret {
            $sentinel => {
                Err(Win32Error { code: 1, msg: "boo hoo".to_string() })
            }
            _ => Ok(ret)
        }
    }
}

fn sum(a: u32, b: u32) -> u32 {
    a + b
}

fn create_socket() -> Win32Result<SOCKET> {
    /*
    unsafe {
        let s = WSASocketW(
            AF_INET.0 as i32,
            SOCK_RAW as i32,
            IPPROTO_ICMP.0,
            None,
            0,
            WSA_FLAG_OVERLAPPED
        );
        if s == INVALID_SOCKET {
            let mut buf: usize = 0;
            let sz_buf = FormatMessageW(
                FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
                None,
                WSAGetLastError().0 as u32,
                0,
                PWSTR::from_raw(&mut buf as *mut usize as *mut u16),
                0,
                None
            );
            let buf_wstr = std::mem::transmute::<usize, *const u16>(buf);
            let buf_wstr = U16CStr::from_ptr(buf_wstr, sz_buf as usize).unwrap();
            println!("ERROR: failed to create socket: {}", buf_wstr.to_string().unwrap())
        }
        Ok(s)
    }
    */
    let x = test!(
        sum(
            7 as u32,
            4 as u32
        ),
        11
    );
    let res = win32!(
        WSASocketW(
            AF_INET.0 as i32,
            SOCK_RAW as i32,
            IPPROTO_ICMP.0,
            None,
            0,
            WSA_FLAG_OVERLAPPED
        ),
        INVALID_SOCKET
    );
    Ok(INVALID_SOCKET)
}
