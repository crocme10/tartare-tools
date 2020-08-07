use chrono::{DateTime, FixedOffset};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use transit_model::Result;

mod apply_rules;

#[derive(Debug, StructOpt)]
#[structopt(name = "apply_rules", about = "Enrich the data of an NTFS.")]
struct Opt {
    /// Input directory.
    #[structopt(short = "i", long = "input", parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// Complementary code rules files.
    #[structopt(short = "c", long = "complementary-code-rules", parse(from_os_str))]
    complementary_code_rules_files: Vec<PathBuf>,

    /// Property rules files.
    #[structopt(short = "p", long = "property-rules", parse(from_os_str))]
    property_rules_files: Vec<PathBuf>,

    /// Object rules file.
    #[structopt(long = "object-rules", parse(from_os_str))]
    object_rules_file: Option<PathBuf>,

    /// Route consolidation configuration.
    #[structopt(long = "routes-consolidation", parse(from_os_str))]
    routes_consolidation_file: Option<PathBuf>,

    /// Output report file path.
    #[structopt(short = "r", long = "report", parse(from_os_str))]
    report: PathBuf,

    /// Output directory.
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

    /// Current datetime.
    #[structopt(
        short = "x",
        long,
        parse(try_from_str),
        default_value = &transit_model::CURRENT_DATETIME
    )]
    current_datetime: DateTime<FixedOffset>,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching apply_rules.");

    let model = apply_rules::apply_rules(
        transit_model::ntfs::read(opt.input)?,
        opt.object_rules_file,
        opt.routes_consolidation_file,
        opt.complementary_code_rules_files,
        opt.property_rules_files,
        opt.report,
    )?;

    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;

    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
