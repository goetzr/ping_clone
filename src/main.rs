use windows::Win32::Networking::WinSock::*;
use windows::Win32::System::Diagnostics::Debug::*;
use windows::core::*;
use widestring::U16CStr;

fn main() {
    let _ = create_socket();
}

fn create_socket() -> std::result::Result<SOCKET, String> {
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
}
