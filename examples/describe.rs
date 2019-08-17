#[macro_use]
extern crate log;

use std::{io::Read, fs::File, error::Error};
use fern::colors::{Color, ColoredLevelConfig};
use aces::{Context, Content, ContentOrigin, CES, sat, AcesError};
use ascesis::CesFile;

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
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout());

    let root_logger = fern::Dispatch::new().chain(console_logger);
    root_logger.apply().unwrap_or_else(|err| eprintln!("[ERROR] {}.", err));

    let args = clap::App::new("Describe")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Ascesis description demo")
        .args_from_usage("[ROOT_PATH] 'path to a script'")
        .get_matches();

    if let Some(root_path) = args.value_of("ROOT_PATH") {
        let ctx = Context::new_toplevel("describe", ContentOrigin::ces_script(&root_path));
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

        let ces = CES::from_content(ctx.clone(), Box::new(ces_file))?;
        debug!("{:?}", ces);

        if !ces.is_coherent() {
            return Err(Box::new(AcesError::CESIsIncoherent(
                ces.get_name().unwrap_or("anonymous").to_owned(),
            )))
        } else {
            let formula = ces.get_formula();

            debug!("Raw {:?}", formula);
            debug!("Formula: {}", formula);

            let mut solver = sat::Solver::new(ctx.clone());
            solver.add_formula(&formula);
            solver.inhibit_empty_solution();

            if let Some(first_solution) = solver.next() {
                debug!("1. Raw {:?}", first_solution);
                println!("1. Solution: {}", first_solution);

                for (count, solution) in solver.enumerate() {
                    debug!("{}. Raw {:?}", count + 2, solution);
                    println!("{}. Solution: {}", count + 2, solution);
                }
            } else if solver.is_sat().is_some() {
                println!("\nStructural deadlock (found no solutions).");
            } else if solver.was_interrupted() {
                warn!("Solving was interrupted");
            } else if let Some(Err(err)) = solver.take_last_result() {
                error!("Solving failed: {}", err);
            } else {
                unreachable!()
            }
        }
    }

    Ok(())
}
