use std::fmt;
use std::mem::MaybeUninit;

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
        write!(f, "({}) {}", self.code, self.msg,)
    }
}

impl std::error::Error for Win32Error {}

type Win32Result<T> = std::result::Result<T, Win32Error>;

fn make_win32_error() -> Win32Error {
    unsafe {
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
        let w32_err = Win32Error {
            code: err.0 as u32,
            msg: buf_str.to_string().unwrap(),
        };
        LocalFree(buf as isize);
        w32_err
    }
}

macro_rules! win32_eq {
    ( $call:expr, $sentinel:pat ) => {{
        unsafe {
            let ret = $call;
            match ret {
                $sentinel => Ok(()),
                _ => Err(make_win32_error()),
            }
        }
    }};
}

macro_rules! win32_ne {
    ( $call:expr, $sentinel:pat ) => {{
        unsafe {
            let ret = $call;
            match ret {
                $sentinel => Err(make_win32_error()),
                _ => Ok(ret),
            }
        }
    }};
}

macro_rules! win32_eq_zero {
    ( $call:expr ) => {
        win32_eq!($call, 0)
    };
}

pub fn wsa_startup() -> Win32Result<()> {
    let mut wsa_data = MaybeUninit::<WSADATA>::uninit();
    let res = win32_eq_zero!(WSAStartup(
        2 << 8 | 2,
        wsa_data.as_mut_ptr() as *mut WSADATA
    ));
    let _ = unsafe { wsa_data.assume_init() };
    res
}

pub fn create_raw_icmp_socket() -> Win32Result<SOCKET> {
    let sock = win32_ne!(
        WSASocketW(
            AF_INET.0 as i32,
            SOCK_RAW as i32,
            IPPROTO_ICMP.0,
            None,
            0,
            WSA_FLAG_OVERLAPPED
        ),
        INVALID_SOCKET
    )?;
    win32_eq_zero!(
        setsockopt(sock, IPPROTO_IP as i32, IP_HDRINCL as i32, Some(&1))
    )?;
    Ok(sock)
}
