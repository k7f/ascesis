use std::error::Error;
use cesar_lang::{Rex, CesarError};

fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::App::new("Rex")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Rule Expression Parsing Demo")
        .args_from_usage("[REX] 'rule expression'")
        .get_matches();

    let spec = args
        .value_of("REX")
        .unwrap_or(r#"{ a => b }"#);  // FIXME random rex

    let rex: Rex = spec.parse().map_err(|err| {
        let message = format!("{}", err);
        let mut lines = message.lines();

        if let Some(line) = lines.next() {
            eprintln!("[ERROR] {}", line);
        }

        for line in lines {
            eprintln!("\t{}", line);
        }

        CesarError::from(err)
    })?;

    println!("Rex: {:?}", rex);

    Ok(())
}
