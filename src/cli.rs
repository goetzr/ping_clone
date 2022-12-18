use windows::Win32::Foundation::*;
use windows::Win32::System::Console::*;

pub unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    println!("Control key {} pressed", ctrl_type);
    match ctrl_type {
        CTRL_C_EVENT => {
            println!("CTRL-C pressed");
            std::process::exit(0);
        }
        CTRL_BREAK_EVENT => {
            // Fn+Ctrl+P on Lenovo laptop.
            println!("CTRL-BREAK pressed");
            true.into()
        }
        _ => false.into()
    }
}