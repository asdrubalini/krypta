use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::io::Write;

pub struct RandomString;

impl RandomString {
    pub fn alphanum_with_rng(rng: &mut impl Rng, len: usize) -> String {
        rng.sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect::<String>()
    }

    pub fn alphanum(len: usize) -> String {
        let mut rng = thread_rng();
        Self::alphanum_with_rng(&mut rng, len)
    }

    pub fn hex_with_rng(rng: &mut impl Rng, len: usize) -> String {
        (0..len)
            .into_iter()
            .map(|_| {
                let random_byte: u8 = rng.gen_range(0..=255);
                format!("{:02x}", random_byte)
            })
            .collect()
    }

    pub fn hex(len: usize) -> String {
        let mut rng = thread_rng();
        Self::hex_with_rng(&mut rng, len)
    }
}

pub fn ask_yes_or_no(question: impl AsRef<str>) {
    let question = question.as_ref();
    print!("{question} y/N ");

    std::io::stdout().lock().flush().unwrap();

    let mut in_buf = String::new();
    std::io::stdin().read_line(&mut in_buf).unwrap();

    in_buf = in_buf.trim().to_lowercase();

    if in_buf.is_empty() || !in_buf.starts_with('y') {
        println!("Stopped.");
        std::process::exit(0);
    }
}
