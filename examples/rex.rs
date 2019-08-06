#[macro_use]
extern crate log;

use std::{fmt, error::Error};
use rand::{thread_rng, Rng};
use fern::colors::{Color, ColoredLevelConfig};
use ascesis::{Axiom, grammar::Grammar, sentence::Generator};

#[derive(Debug)]
struct RexError(String);

impl fmt::Display for RexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for RexError {}

fn random_phrase(axiom: &Axiom) -> Result<String, Box<dyn Error>> {
    let grammar = Grammar::of_ascesis();
    trace!("{:?}", grammar);

    let generator = Generator::new(&grammar);

    let mut all_phrases: Vec<_> = generator.rooted(axiom.symbol())?.iter().collect();

    if all_phrases.is_empty() {
        Err(Box::new(RexError(format!("Random phrase generation failed for {:?}.", axiom))))
    } else {
        let mut rng = thread_rng();
        let result = all_phrases.remove(rng.gen_range(0, all_phrases.len()));

        Ok(result)
    }
}

fn get_axiom_and_phrase(maybe_arg: Option<&str>) -> Result<(Axiom, String), Box<dyn Error>> {
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
        let phrase = random_phrase(&axiom)?;
        info!("{:?} generated \"{}\"", axiom, phrase);

        Ok((axiom, phrase))
    } else {
        let arg = maybe_arg.unwrap();
        let axiom = Axiom::guess_from_phrase(arg);
        let phrase = arg.to_owned();
        info!("{:?} guessed from \"{}\"", axiom, phrase);

        Ok((axiom, phrase))
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
        .about("Ascesis parsing demo")
        .args_from_usage("[SENTENCE_OR_AXIOM] 'sentence or axiom to parse (default: \'Rex\')'")
        .get_matches();

    let maybe_arg = args.value_of("SENTENCE_OR_AXIOM");
    let (axiom, phrase) = get_axiom_and_phrase(maybe_arg)?;

    let result = axiom.parse(phrase)?;
    println!("{:?}", result);

    Ok(())
}
