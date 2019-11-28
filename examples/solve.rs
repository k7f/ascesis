#[macro_use]
extern crate log;

use std::{io::Read, fs::File, error::Error};
use fern::colors::{Color, ColoredLevelConfig};
use aces::{Context, Contextual, Content, ContentOrigin, CEStructure};
use ascesis::CesFile;

fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::App::new("solve")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Ascesis solving demo")
        .args_from_usage(
            "[ROOT_PATH]      'path to a script'
             -v, --verbose... 'level of verbosity'",
        )
        .get_matches();

    let log_level = match args.occurrences_of("verbose") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

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
        .level(log_level)
        .chain(std::io::stdout());

    let root_logger = fern::Dispatch::new().chain(console_logger);
    root_logger.apply().unwrap_or_else(|err| eprintln!("[ERROR] {}.", err));

    if let Some(root_path) = args.value_of("ROOT_PATH") {
        let ctx = Context::new_toplevel("solve", ContentOrigin::ces_script(&root_path));
        let mut fp = File::open(root_path)?;
        let mut script = String::new();
        fp.read_to_string(&mut script)?;

        let mut ces_file = CesFile::from_script(script)?;
        ces_file.set_root_name("Main")?;
        if let Some(title) = ces_file.get_vis_name("title") {
            info!("Using '{}' as the root structure: \"{}\"", ces_file.get_name().unwrap(), title);
        } else {
            info!("Using '{}' as the root structure", ces_file.get_name().unwrap());
        }

        ces_file.compile(&ctx)?;
        debug!("{:?}", ces_file);

        let mut ces = CEStructure::new(&ctx).with_content(Box::new(ces_file))?;
        debug!("{:?}", ces);

        ces.solve()?;

        if let Some(fset) = ces.get_firing_set() {
            println!("Firing components:");

            for (i, fc) in fset.as_slice().iter().enumerate() {
                println!("{}. {}", i + 1, fc.with(&ctx));
            }
        }
    } else {
        println!("{}", args.usage());
    }

    Ok(())
}
