use crate::{args::Args, MainError};
use log::{warn, Level, LevelFilter};
use simplelog::{
    format_description, Color, ColorChoice, ConfigBuilder, LevelPadding, TermLogger, TerminalMode,
};

// Use the same format as simple_logger
const TIMESTAMP_FORMAT_OFFSET: &[simplelog::FormatItem<'_>] = format_description!(
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory]:[offset_minute]"
);

pub fn init_logger(args: &Args) -> Result<(), MainError> {
    TermLogger::init(
        match (args.quiet, args.verbose) {
            (true, _) => LevelFilter::Error,
            (false, 0) => LevelFilter::Info,
            (false, 1) => LevelFilter::Debug,
            (false, _) => LevelFilter::Trace,
        },
        ConfigBuilder::new()
            .set_level_color(Level::Debug, Some(Color::Magenta))
            .set_level_color(Level::Trace, None)
            .set_level_padding(LevelPadding::Right)
            .set_target_level(LevelFilter::Off)
            .set_thread_level(LevelFilter::Off)
            .set_time_format_custom(TIMESTAMP_FORMAT_OFFSET)
            .set_time_offset_to_local()
            .map_err(|_| MainError::FailedLoggerTimezones)?
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    if args.verbose > 3 {
        warn!("Okay, it's time to stop. It won't get more verbose than this.")
    }

    Ok(())
}
