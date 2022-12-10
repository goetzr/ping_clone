use std::fmt;
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;

use widestring::U16CStr;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::NetworkManagement::Dns::*;
use windows::Win32::NetworkManagement::IpHelper::*;
use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Memory::*;

#[derive(Debug)]
pub enum Error {
    CreateSocket(Win32Error),
    SetIpHdrSockOpt(Win32Error),
    OpenIcmpHandle(Win32Error),
    SendIcmpEcho(Win32Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::CreateSocket(e) => write!(f, "failed to create the socket: {e}"),
            Error::SetIpHdrSockOpt(e) => {
                write!(f, "failed to set the IP_HDRINCL socket option: {e}")
            }
            Error::OpenIcmpHandle(e) => write!(f, "failed to open an ICMP handle: {e}"),
            Error::SendIcmpEcho(e) => write!(f, "failed to send the ICMP echo request: {e}"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

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

impl Win32Error {
    fn new<S: Into<String>>(code: u32, msg: S) -> Win32Error {
        Win32Error {
            code,
            msg: msg.into(),
        }
    }
}

type Win32Result<T> = std::result::Result<T, Win32Error>;

fn make_win32_error_with_code(err: u32) -> Win32Error {
    unsafe {
        let mut buf: usize = 0;
        let sz_buf = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            None,
            err,
            0,
            PWSTR::from_raw(&mut buf as *mut usize as *mut u16),
            0,
            None,
        );
        let buf_str = std::mem::transmute::<usize, *const u16>(buf);
        let buf_str = U16CStr::from_ptr(buf_str, sz_buf as usize).unwrap();
        let w32_err = Win32Error {
            code: err,
            msg: buf_str.to_string().unwrap(),
        };
        LocalFree(buf as isize);
        w32_err
    }
}

fn make_win32_error() -> Win32Error {
    make_win32_error_with_code(unsafe { GetLastError().0 as u32 })
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

macro_rules! win32_ne_zero {
    ( $call:expr ) => {
        win32_ne!($call, 0)
    };
}

// macro_rules! win32_is_ok {
//     ( $call:expr ) => {{
//         unsafe {
//             let ret = $call;
//             match ret.ok() {
//                 Err(e) => Err(make_win32_error_with_code(e.0)),
//                 _ => Ok(()),
//             }
//         }
//     }};
// }

pub fn wsa_startup() -> Win32Result<()> {
    let mut wsa_data = MaybeUninit::<WSADATA>::uninit();
    let res = win32_eq_zero!(WSAStartup(
        2 << 8 | 2,
        wsa_data.as_mut_ptr() as *mut WSADATA
    ));
    let _ = unsafe { wsa_data.assume_init() };
    res
}

fn ascii_to_wide(data: &str) -> Vec<u16> {
    let mut result = Vec::new();
    for ch in data.chars() {
        if !ch.is_ascii() {
            panic!("non-ascii character");
        }
        result.push(ch as u16);
    }
    result.push(0); // NULL terminator
    result
}

pub fn resolve_hostname(hostname: &str) -> Win32Result<String> {
    let hostname = ascii_to_wide(hostname);
    let hostname = PCWSTR::from_raw(hostname.as_ptr());
    unsafe {
        let mut query_results: usize = 0;
        DnsQuery_W(
            hostname,
            DNS_TYPE_A,
            DNS_QUERY_STANDARD,
            None,
            Some(std::mem::transmute::<&usize, *mut *mut DNS_RECORDA>(&query_results)),
            None,
        )
        .ok()
        .map_err(|e| Win32Error {
            code: e.code().0 as u32,
            msg: e.message().to_string(),
        })?;
        // TODO: Call DnsRecordListFree
    }
    unimplemented!();
}

// pub fn send_ping(dst_addr: IPV4Addr, ttl: u8) -> Result<()> {
//     let icmp_handle = unsafe {
//         IcmpCreateFile().map_err(|e| {
//             Error::OpenIcmpHandle(Win32Error::new(e.code().0 as u32, e.message().to_string()))
//         })?
//     };
//     let ip_options = IP_OPTION_INFORMATION { Ttl: ttl, Tos: 0, Flags: 0, OptionsSize: 0, OptionsData: std::ptr::null() as *mut u8 };
//     win32_ne_zero!(
//         IcmpSendEcho(icmp_handle, IPV4Addr::new(192, 168, 1, 1).into(), )
//     )
//     Ok(())
// }

mod test {
    #[test]
    fn ascii_to_wide_ok() {
        let result = super::ascii_to_wide("test");
        let expected = vec![b't' as u16, b'e' as u16, b's' as u16, b't' as u16, 0];
        assert_eq!(result, expected);
    }
}
