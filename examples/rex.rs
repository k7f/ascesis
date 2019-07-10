use std::error::Error;
use cesar_lang::{Rex, ParsingError, CesarError, grammar::Grammar, sentence::Generator};

fn random_spec(axiom: &str) -> Result<String, Box<dyn Error>> {
    let grammar = Grammar::of_cesar();
    // println!("{:?}", grammar);

    let mut generator = Generator::new().with_grammar(&grammar);

    generator.set_axiom(&grammar, axiom)?;
    // println!("Axiom: <{}>", axiom);

    Ok(generator.emit(&grammar))
}

fn process_parsing_error(err: ParsingError) -> CesarError {
    let message = format!("{}", err);
    let mut lines = message.lines();

    if let Some(line) = lines.next() {
        eprintln!("[ERROR] {}", line);
    }

    for line in lines {
        eprintln!("\t{}", line);
    }

    CesarError::from(err)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::App::new("Rex")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Rule Expression Parsing Demo")
        .args_from_usage("[REX] 'rule expression'")
        .get_matches();

    let spec = {
        if let Some(axiom) = {
            if let Some(arg) = args.value_of("REX") {
                if arg.trim().starts_with('{') {
                    None
                } else {
                    Some(arg)
                }
            } else {
                Some("Rex")
            }
        } {
            let spec = random_spec(axiom)?;
            println!("<{}> is \"{}\"", axiom, spec);

            if axiom == "Rex" {
                spec
            } else {
                format!("{{ {} }}", spec)
            }
        } else {
            args.value_of("REX").unwrap().to_owned()
        }
    };

    let rex: Rex = spec.parse().map_err(process_parsing_error)?;
    println!("Rex: {:?}", rex);

    Ok(())
}
