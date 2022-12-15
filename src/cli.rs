use clap::{Parser, Subcommand};
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
            println!("CTRL-BREAK pressed");
            true.into()
        }
        _ => false.into()
    }
}

// #[derive(Parser)]
// //#[command(author, version, about, long_about = None)]
// struct Cli {
    
// }