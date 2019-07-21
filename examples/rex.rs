#[macro_use]
extern crate log;

use std::{fmt, error::Error};
use rand::{thread_rng, Rng};
use fern::colors::{Color, ColoredLevelConfig};
use cesar_lang::{Axiom, grammar::Grammar, sentence::Generator};

#[derive(Debug)]
struct RexError(String);

impl fmt::Display for RexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for RexError {}

fn random_spec(axiom: &Axiom) -> Result<String, Box<dyn Error>> {
    let grammar = Grammar::of_cesar();
    debug!("{:?}", grammar);

    let generator = Generator::new(&grammar);

    let mut all_specs: Vec<_> = generator.rooted(axiom.symbol())?.iter().collect();

    if all_specs.is_empty() {
        Err(Box::new(RexError(format!("Random spec generation failed for {:?}.", axiom))))
    } else {
        let mut rng = thread_rng();
        let result = all_specs.remove(rng.gen_range(0, all_specs.len()));

        Ok(result)
    }
}

fn get_axiom_and_spec(maybe_arg: Option<&str>) -> Result<(Axiom, String), Box<dyn Error>> {
    if let Some(axiom) = {
        if let Some(arg) = maybe_arg {
            let arg = arg.trim();
            if arg.starts_with(|c: char| c.is_uppercase()) {
                Axiom::from_known_symbol(arg)
            } else {
                None
            }
        } else {
            Axiom::from_known_symbol("Rex")
        }
    } {
        let spec = random_spec(&axiom)?;
        println!("{:?} is \"{}\"", axiom, spec);

        Ok((axiom, spec))

    } else {
        let arg = maybe_arg.unwrap();
        let axiom = Axiom::guess_from_spec(arg);
        let spec = arg.to_owned();
        
        Ok((axiom, spec))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let colors = ColoredLevelConfig::new()
        .trace(Color::Blue)
        .debug(Color::Yellow)
        .info(Color::Green)
        .warn(Color::Magenta)
        .error(Color::Red);

    let console_logger = fern::Dispatch::new()
        .format(move |out, message, record| match record.level() {
            log::Level::Info => out.finish(format_args!("{}.", message)),
            log::Level::Warn | log::Level::Debug => {
                out.finish(format_args!("[{}]\t{}.", colors.color(record.level()), message))
            }
            _ => out.finish(format_args!(
                "[{}]\t\x1B[{}m{}.\x1B[0m",
                colors.color(record.level()),
                colors.get_color(&record.level()).to_fg_str(),
                message
            )),
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout());

    let root_logger = fern::Dispatch::new().chain(console_logger);
    root_logger.apply().unwrap_or_else(|err| eprintln!("[ERROR] {}.", err));

    let args = clap::App::new("Rex")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Rule Expression Parsing Demo")
        .args_from_usage("[REX] 'rule expression'")
        .get_matches();

    let maybe_arg = args.value_of("REX");
    let (axiom, spec) = get_axiom_and_spec(maybe_arg)?;

    let result = axiom.parse(spec)?;
    println!("{:?}", result);

    Ok(())
}
