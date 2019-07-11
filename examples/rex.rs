use std::{fmt, error::Error};
use rand::{thread_rng, Rng};
use cesar_lang::{Rex, ParsingError, CesarError, grammar::Grammar, sentence::Generator};

#[derive(Debug)]
struct RexError(String);

impl fmt::Display for RexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for RexError {}

fn random_spec(axiom: &str) -> Result<String, Box<dyn Error>> {
    let grammar = Grammar::of_cesar();
    let generator = Generator::new(&grammar);

    let mut all_specs: Vec<_> = generator.rooted(axiom)?.iter().collect();

    if all_specs.is_empty() {
        Err(Box::new(RexError(format!("Random spec generation failed for axiom \"{}\".", axiom))))
    } else {
        let mut rng = thread_rng();
        let result = all_specs.remove(rng.gen_range(0, all_specs.len()));

        Ok(result)
    }
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
