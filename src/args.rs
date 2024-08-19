use duration_string::DurationString;
use gumdrop::Options;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum Trigger {
    Push,
}

impl FromStr for Trigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "push" => Ok(Trigger::Push),
            s => Err(format!("cannot parse {s}, valid values: push")),
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
    #[options(long = "once", no_short)]
    pub once: bool,

    /// The trigger on which to run.
    #[options(default = "push")]
    pub trigger: Trigger,

    /// Refreshes the repo with this delay.
    ///
    /// Can be a number postfixed with s(econd), m(inutes), h(ours), d(ays)
    #[options(long = "every", default = "1m")]
    pub delay: DurationString,

    /// Set the path for an ssh-key to be used when pulling.
    #[options(short = 'i', long = "ssh-key")]
    pub ssh_key: Option<String>,

    /// Set the username for git to be used when pulling with HTTPS.
    #[options(no_short, meta = "USER")]
    pub git_username: Option<String>,

    /// Set the token for git to be used when pulling with HTTPS.
    #[options(no_short, meta = "TOKEN")]
    pub git_token: Option<String>,

    /// Add this line to the known_hosts file to be created (e.g. "example.com ssh-ed25519 AAAAC3...").
    #[options(no_short, meta = "HOST")]
    pub git_known_host: Option<String>,

    /// Runs an HTTP server on the URL, which allows to trigger by calling it.
    #[options(no_short)]
    pub http: Option<String>,

    /// Increase verbosity, can be set multiple times (-v debug, -vv tracing).
    #[options(count)]
    pub verbose: u8,

    /// Only print error messages.
    #[options()]
    pub quiet: bool,

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
