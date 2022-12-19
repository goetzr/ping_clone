use std::alloc::Layout;
use std::ffi::c_void;
use std::fmt;
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::NetworkManagement::Dns::*;
use windows::Win32::NetworkManagement::IpHelper::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::Win32::System::Memory::*;

#[derive(Debug)]
pub enum Error {
    ResolveIpAddr(Win32Error),
    OpenIcmpHandle(Win32Error),
    SendIcmpEcho(Win32Error),
    SetConsoleHandler(Win32Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ResolveIpAddr(e) => {
                write!(f, "failed to resolve the hostname to an IP address: {e}")
            }
            Error::OpenIcmpHandle(e) => write!(f, "failed to open an ICMP handle: {e}"),
            Error::SendIcmpEcho(e) => write!(f, "failed to send the ICMP echo request: {e}"),
            Error::SetConsoleHandler(e) => write!(f, "failed to set the console handler: {e}"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Win32Error {
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
        let _sz_buf = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
            None,
            err,
            0,
            PWSTR::from_raw(&mut buf as *mut usize as *mut u16),
            0,
            None,
        );
        let buf_str = std::mem::transmute::<usize, *const u16>(buf);
        let w32_err = Win32Error {
            code: err,
            msg: wide_to_utf8(buf_str),
        };
        LocalFree(buf as isize);
        w32_err
    }
}

fn make_win32_error() -> Win32Error {
    make_win32_error_with_code(unsafe { GetLastError().0 as u32 })
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

macro_rules! win32_ne_zero {
    ( $call:expr ) => {
        win32_ne!($call, 0)
    };
}

macro_rules! win32_is_true {
    ( $call:expr ) => {
        win32_eq!($call, true)
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

fn wide_to_utf8(data: *const u16) -> String {
    unsafe {
        let mut null_term = data;
        while *null_term != 0 {
            null_term = null_term.add(1);
        }
        let len = null_term.offset_from(data);
        String::from_utf16(std::slice::from_raw_parts(data, len as usize)).expect("invalid UTF-8)")
    }
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

pub fn icmp_create() -> Result<IcmpHandle> {
    unsafe {
        IcmpCreateFile().map_err(|e| {
            Error::OpenIcmpHandle(Win32Error::new(e.code().0 as u32, e.message().to_string()))
        })
    }
}

fn build_ping_request_data(ttl: u8) -> (Vec<u8>, IP_OPTION_INFORMATION) {
    let data: Vec<u8> = (0..32).into_iter().map(|n| 'A' as u8 + n % 26).collect();
    let sz_data = data.len() * std::mem::size_of::<u8>();
    let options = IP_OPTION_INFORMATION {
        Ttl: ttl,
        Tos: 0,
        Flags: 0,
        OptionsSize: 0,
        OptionsData: std::ptr::null::<u8>() as *mut u8,
    };
    (data, options)
}

fn build_ping_reply_buffer(sz_request_data: usize) -> (*mut u8, Layout) {
    let sz_buf = std::mem::size_of::<ICMP_ECHO_REPLY>() + sz_request_data;
    let layout = Layout::from_size_align(sz_buf, std::mem::align_of::<ICMP_ECHO_REPLY>())
        .expect("reply buffer layout");
    let buf = unsafe { std::alloc::alloc(layout) };
    (buf, layout)
}

pub fn send_ping(
    icmp_handle: IcmpHandle,
    dst_addr: Ipv4Addr,
    ttl: u8,
    timeout: u32,
) -> Result<ICMP_ECHO_REPLY> {
    let (request_data, request_options) = build_ping_request_data(ttl);
    let (reply_buf, reply_buf_layout) = build_ping_reply_buffer(request_data.len());

    // TODO: Call IcmpSendEcho2Ex
    let num_replies = win32_ne_zero!(IcmpSendEcho(
        icmp_handle,
        Into::<u32>::into(dst_addr).swap_bytes(),
        request_data.as_ptr() as *const c_void,
        request_data.len() as u16,
        Some(&request_options as *const IP_OPTION_INFORMATION),
        reply_buf as *mut c_void,
        reply_buf_layout.size() as u32,
        timeout
    ))
    .map_err(|e| Error::SendIcmpEcho(e))?;

    let reply = unsafe { *(reply_buf as *const ICMP_ECHO_REPLY) };
    unsafe { std::alloc::dealloc(reply_buf, reply_buf_layout) };
    Ok(reply)
}

pub fn set_console_ctrl_handler(handler: unsafe extern "system" fn(u32) -> BOOL) -> Result<()> {
    win32_is_true!(
        SetConsoleCtrlHandler(Some(handler), true).as_bool()
    ).map_err(|e| Error::SetConsoleHandler(e))?;
    Ok(())
}
