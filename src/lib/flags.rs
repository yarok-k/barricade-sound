use clap::Parser;
use std::ffi::OsString;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Переключатель: запустить, если не запущено; завершить, если уже запущено
    #[arg(short, long)]
    pub switch: bool,
}

/// Возвращает true, если передан -s / --switch.
pub fn parse(args: Vec<OsString>) -> Result<bool, clap::Error> {
    Ok(Args::try_parse_from(args)?.switch)
}
