use std::ffi::c_void;
use std::fmt;
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::NetworkManagement::Dns::*;
use windows::Win32::NetworkManagement::IpHelper::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Memory::*;

#[derive(Debug)]
pub enum Error {
    ResolveIpAddr(Win32Error),
    OpenIcmpHandle(Win32Error),
    SendIcmpEcho(Win32Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ResolveIpAddr(e) => {
                write!(f, "failed to resolve the hostname to an IP address: {e}")
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

// macro_rules! win32_eq {
//     ( $call:expr, $sentinel:pat ) => {{
//         unsafe {
//             let ret = $call;
//             match ret {
//                 $sentinel => Ok(()),
//                 _ => Err(make_win32_error()),
//             }
//         }
//     }};
// }

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

// macro_rules! win32_eq_zero {
//     ( $call:expr ) => {
//         win32_eq!($call, 0)
//     };
// }

macro_rules! win32_ne_zero {
    ( $call:expr ) => {
        win32_ne!($call, 0)
    };
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

pub fn resolve_hostname(hostname: &str) -> Result<Ipv4Addr> {
    let hostname = ascii_to_wide(hostname);
    let hostname = PCWSTR::from_raw(hostname.as_ptr());
    unsafe {
        let mut query_results = MaybeUninit::<&DNS_RECORDA>::uninit();
        DnsQuery_W(
            hostname,
            DNS_TYPE_A,
            DNS_QUERY_STANDARD,
            None,
            Some(query_results.as_mut_ptr() as *mut *mut DNS_RECORDA),
            None,
        )
        .ok()
        .map_err(|e| {
            Error::ResolveIpAddr(Win32Error {
                code: e.code().0 as u32,
                msg: e.message().to_string(),
            })
        })?;

        let query_results = query_results.assume_init();
        let ip_addr = Ipv4Addr::from(query_results.Data.A.IpAddress.swap_bytes());

        DnsFree(
            Some(query_results as *const DNS_RECORDA as *const c_void),
            DnsFreeRecordList,
        );

        Ok(ip_addr)
    }
}

pub fn send_ping(dst_addr: Ipv4Addr, ttl: u8, timeout: u32) -> Result<ICMP_ECHO_REPLY> {
    let icmp_handle = unsafe {
        IcmpCreateFile().map_err(|e| {
            Error::OpenIcmpHandle(Win32Error::new(e.code().0 as u32, e.message().to_string()))
        })?
    };

    let request_data: Vec<u8> = (0..32).into_iter().map(|n| 65 + n %26).collect();
    let sz_request_data = request_data.len() * std::mem::size_of::<u8>();
    let request_options = IP_OPTION_INFORMATION {
        Ttl: ttl,
        Tos: 0,
        Flags: 0,
        OptionsSize: 0,
        OptionsData: std::ptr::null::<u8>() as *mut u8,
    };
    let sz_reply_buf = std::mem::size_of::<ICMP_ECHO_REPLY>() + sz_request_data;
    let reply_buf_layout = std::alloc::Layout::from_size_align(
        std::mem::size_of::<ICMP_ECHO_REPLY>() + sz_request_data,
        std::mem::align_of::<ICMP_ECHO_REPLY>(),
    )
    .expect("ICMP_ECHO_REPLY layout");
    let reply_buf = unsafe { std::alloc::alloc(reply_buf_layout) };

    // TODO: Call IcmpSendEcho2Ex
    let num_replies = win32_ne_zero!(IcmpSendEcho(
        icmp_handle,
        Into::<u32>::into(dst_addr).swap_bytes(),
        request_data.as_ptr() as *const c_void,
        sz_request_data as u16,
        Some(&request_options as *const IP_OPTION_INFORMATION),
        reply_buf as *mut c_void,
        sz_reply_buf as u32,
        timeout * 1000
    ))
    .map_err(|e| Error::SendIcmpEcho(e))?;

    assert_eq!(num_replies, 1);
    let reply = unsafe { *(reply_buf as *const ICMP_ECHO_REPLY) };
    unsafe { std::alloc::dealloc(reply_buf, reply_buf_layout) };

    Ok(reply)
}

mod test {
    #[test]
    fn ascii_to_wide_ok() {
        let result = super::ascii_to_wide("test");
        let expected = vec![b't' as u16, b'e' as u16, b's' as u16, b't' as u16, 0];
        assert_eq!(result, expected);
    }
}
