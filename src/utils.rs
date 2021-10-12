use std::process::exit;

pub fn die(error: &str) -> ! {
    if error.len() > 0 {
        eprintln!("{}", error);
    }
    exit(-1);
}
