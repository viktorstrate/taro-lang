use std::{io::BufRead, process::exit};

use taro::error_message::ErrorMessage;

fn main() -> std::io::Result<()> {
    let mut input = std::io::stdin()
        .lock()
        .lines()
        .collect::<Result<Vec<String>, _>>()?;

    input.iter_mut().for_each(|line| *line += "\n");
    let input = input.into_iter().collect::<String>();

    if let Err(err) = taro::transpile(&mut std::io::stdout(), &input) {
        err.format_err(&mut std::io::stderr(), ())?;
        exit(1);
    }

    Ok(())
}
