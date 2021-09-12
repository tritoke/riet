#![feature(
    derive_default_enum,
    stmt_expr_attributes,
    option_result_contains,
    never_type
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

    /// Enables program tracing
    #[structopt(short, long)]
    trace: bool,

    /// The name of the piet program to interpret
    #[structopt(parse(from_os_str))]
    file_name: PathBuf,
}

fn main() -> anyhow::Result<!> {
    let opt = Opt::from_args();

    if opt.trace {
        simple_logger::init_with_level(log::Level::Trace).unwrap();
    } else {
        simple_logger::init_with_level(log::Level::Warn).unwrap();
    }

    let img = ImageReader::open(opt.file_name)?.decode()?;

    let program = Program::new_from_imagebuffer(&img.to_rgb8(), opt.codel_size);

    let mut interpreter = program.into_interpreter();

    interpreter.run()?;
}
