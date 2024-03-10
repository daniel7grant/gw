use duration_string::DurationString;
use gumdrop::Options;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum Trigger {
    Push,
    Tag(String),
}

impl FromStr for Trigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((trigger_name, value)) = s.split_once(':') {
            match trigger_name {
                "tag" => Ok(Trigger::Tag(String::from(value))),
                s => Err(format!("cannot parse {s}, valid values: push, tag:regex")),
            }
        } else {
            match s {
                "push" => Ok(Trigger::Push),
                s => Err(format!("cannot parse {s}, valid values: push, tag:regex")),
            }
        }
    }
}

/// Watch a repository for changes and run scripts when it happens.
#[derive(Debug, Options)]
pub struct Args {
    /// The git repository to watch.
    #[options(free)]
    pub directory: Option<String>,

    /// The script to run on changes, you can define multiple times.
    ///
    /// If there are no scripts given, it will only pull.
    #[options(long = "script")]
    pub scripts: Vec<String>,

    /// Try to pull only once. Useful for cronjobs.
    #[options()]
    pub once: bool,

    /// The trigger on which to run.
    ///
    /// Can be either "push" to trigger on every change (default)
    /// or "tag:regex" to trigger on every tag matching a regex.
    #[options(default = "push")]
    pub trigger: Trigger,

    /// Refreshes the repo with this delay.
    ///
    /// Can be a number postfixed with s(econd), m(inutes), h(ours), d(ays)
    #[options(long = "every", default = "1m")]
    pub delay: DurationString,

    /// Runs an HTTP server on the URL, which allows to trigger by calling it.
    #[options(no_short)]
    pub http: Option<String>,

    /// Increase verbosity, can be set multiple times (-v debug, -vv tracing)
    #[options(count)]
    pub verbose: u8,

    /// Print the current version.
    #[options(short = "V")]
    pub version: bool,

    /// Print this help.
    #[options()]
    pub help: bool,
}

pub fn parse_args() -> Args {
    Args::parse_args_default_or_exit()
}
