use std::io::{self, BufRead, Write};

#[allow(dead_code)]

pub fn print_prompt_and_read_input() -> String {
    print!("-> ");

    io::stdout().flush().unwrap();

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap();

    line
}
