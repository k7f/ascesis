#[macro_use]
extern crate log;

use std::{fmt, error::Error};
use rand::{thread_rng, Rng};
use fern::colors::{Color, ColoredLevelConfig};
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
    debug!("{:?}", grammar);

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
        error!("{}", line);
    }

    for line in lines {
        error!("\t{}", line);
    }

    CesarError::from(err)
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
