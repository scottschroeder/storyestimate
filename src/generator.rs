use rand::{self, Rng};

pub fn session_id() -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .filter(|&c| c < 'A' || c > 'Z')
        .take(5)
        .collect::<String>()
}

pub fn authtoken() -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(25)
        .collect::<String>()
}
