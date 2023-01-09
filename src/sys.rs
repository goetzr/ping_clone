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
    ResolveIpAddr(wp::error::Error),
    OpenIcmpHandle(Win32Error),
    SendIcmpEcho(Win32Error),
    SetConsoleHandler(Win32Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ResolveIpAddr(e)=> {
                write!(f, "failed to resolve hostname to an IP address: {}", e)
            }
            Error::OpenIcmpHandle(e) => write!(f, "failed to open an ICMP handle: {e}"),
            Error::SendIcmpEcho(e) => write!(f, "failed to send the ICMP echo request: {e}"),
            Error::SetConsoleHandler(e) => write!(f, "failed to set the console handler: {e}"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

pub fn resolve_hostname(hostname: &str) -> Result<Ipv4Addr> {
    let hostname_utf16 = wp::string::utf8_to_utf16(hostname);
    let mut query_results = MaybeUninit::<&DNS_RECORDA>::uninit();
    unsafe {
        DnsQuery_W(
            PCWSTR::from_raw(hostname_utf16.as_ptr()),
            DNS_TYPE_A,
            DNS_QUERY_STANDARD,
            None,
            Some(query_results.as_mut_ptr() as *mut *mut DNS_RECORDA),
            None,
        )
        .ok()
        .map_err(|e| {
            Error::ResolveIpAddr(wp::error::Error::from_win_error(e))
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
