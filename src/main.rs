mod sys;

pub fn main() {
    // TODO: Call WSAStartup!
    
    match sys::create_socket() {
        Ok(_) => println!("success"),
        Err(e) => println!("ERROR: {e}"),
    }
}