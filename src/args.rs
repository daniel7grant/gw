use duration_string::DurationString;
use gumdrop::Options;
use gw_bin::checks::git::GitTriggerArgument;
use std::{env, str::FromStr};

#[derive(Clone, Debug)]
pub enum TriggerArgument {
    Push,
    Tag(String),
}

impl FromStr for TriggerArgument {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "push" => Ok(TriggerArgument::Push),
            "tag" => Ok(TriggerArgument::Tag("*".to_string())),
            s if s.starts_with("tag:") => Ok(TriggerArgument::Tag(
                s.trim_start_matches("tag:").to_string(),
            )),
            s => Err(format!("cannot parse {s}, valid values: push, tag, tag:prefix")),
        }
    }
}

impl From<TriggerArgument> for GitTriggerArgument {
    fn from(value: TriggerArgument) -> Self {
        match value {
            TriggerArgument::Push => GitTriggerArgument::Push,
            TriggerArgument::Tag(t) => GitTriggerArgument::Tag(t),
        }
    }
}

/// Watch a repository for changes and run scripts when it happens.
#[derive(Debug, Options)]
pub struct Args {
    /// The git repository to watch.
    #[options(free)]
    pub directory: Option<String>,

    /// A script to run on changes, you can define multiple times.
    ///
    /// If there are no scripts given, it will only pull.
    #[options(long = "script", meta = "SCRIPT")]
    pub scripts: Vec<String>,

    /// Run a script in a shell.
    #[options(short = "S", no_long, meta = "SCRIPT")]
    pub scripts_with_shell: Vec<String>,

    /// A background process that will be restarted on change.
    #[options(meta = "PROCESS")]
    pub process: Option<String>,

    /// Run a background process in a shell.
    #[options(short = "P", no_long, meta = "PROCESS")]
    pub process_with_shell: Option<String>,

    /// Try to pull only once. Useful for cronjobs.
    #[options(long = "once", no_short)]
    pub once: bool,

    /// The trigger on which to run (can be `push`, `tag` or `tag:pattern`).
    ///
    /// The options are:
    /// - `push`: update on every commit,
    /// - `tag`: update on every tag on this branch,
    /// - `tag:pattern`: update on tags matching the glob.
    #[options(no_short, long = "on", default = "push")]
    pub trigger: TriggerArgument,

    /// Refreshes the repo with this interval.
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

    /// The number of times to retry the background process in case it fails. By default 0 for no retries.
    #[options(no_short, meta = "N")]
    pub process_retries: Option<u32>,

    /// The stop signal to give the background process. Useful for graceful shutdowns. By default SIGINT. (Only supported on *NIX)
    #[options(no_short, meta = "SIGNAL")]
    pub stop_signal: Option<String>,

    /// The timeout to wait before killing for the background process to shutdown gracefully. By default 10s.
    #[options(no_short, meta = "TIMEOUT")]
    pub stop_timeout: Option<DurationString>,

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

#[derive(Debug)]
pub enum ArgAction {
    Process(String, bool),
    Script(String, bool),
}

pub fn parse_args() -> (Args, Vec<ArgAction>) {
    let args = Args::parse_args_default_or_exit();

    // We have to maintain positionality between different flags
    let arg_actions = env::args()
        .skip(2)
        .filter_map(|arg| {
            if args.process.as_ref() == Some(&arg) {
                Some(ArgAction::Process(arg, false))
            } else if args.process_with_shell.as_ref() == Some(&arg) {
                Some(ArgAction::Process(arg, true))
            } else if args.scripts.contains(&arg) {
                Some(ArgAction::Script(arg, false))
            } else if args.scripts_with_shell.contains(&arg) {
                Some(ArgAction::Script(arg, true))
            } else {
                None
            }
        })
        .collect();

    (args, arg_actions)
}
