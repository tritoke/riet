#![feature(
    derive_default_enum,
    stmt_expr_attributes,
    never_type,
    array_windows,
)]

use image::io::Reader as ImageReader;
use std::path::PathBuf;
use structopt::StructOpt;

mod program;
use program::Program;

mod interpreter;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "piet interpreter",
    about = "Interprets programs written in the piet graphical programming language."
)]
struct Opt {
    /// Set the codel size to use to interpret the program
    #[structopt(short, long)]
    codel_size: u32,

    /// Enables trace log level
    #[structopt(short, long)]
    trace: bool,

    /// Enables info log level
    #[structopt(short, long)]
    info: bool,

    /// The maxiumum number of steps the interpreter will take
    #[structopt(short, long)]
    max_steps: Option<usize>,

    /// The name of the piet program to interpret
    #[structopt(parse(from_os_str))]
    file_name: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();

    let log_level = if opt.trace {
        log::Level::Trace
    } else if opt.info {
        log::Level::Info
    } else {
        log::Level::Warn
    };

    simple_logger::init_with_level(log_level)?;

    let img = ImageReader::open(opt.file_name)?.decode()?;

    let program = Program::new_from_imagebuffer(&img.to_rgb8(), opt.codel_size);

    let mut interpreter = program.into_interpreter();

    if let Some(max_steps) = opt.max_steps {
        interpreter.run_until(max_steps)?;

        Ok(())
    } else {
        interpreter.run()?;
    }
}
