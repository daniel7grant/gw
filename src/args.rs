use clap::{Parser, Subcommand};
use duration_string::DurationString;
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
                s => Err(format!("Cannot parse {s}, valid values: push, tag:regex")),
            }
        } else {
            match s {
                "push" => Ok(Trigger::Push),
                s => Err(format!("Cannot parse {s}, valid values: push, tag:regex")),
            }
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Check for updates once
    ///
    /// Suitable for adding to crontabs
    #[command(alias = "cron")]
    Run {
        /// The git repository to look at
        directory: String,
        /// The script to run on changes
        #[arg(short, long)]
        scripts: Vec<String>,
    },
    /// Run in the foreground and watch for updates
    ///
    /// Suitable for running as a daemon (e.g. systemd unit)
    Watch {
        /// The git repository to watch.
        directory: String,
        /// The script to run on changes, you can define multiple times.
        /// 
        /// If there are no scripts given, it will only pull.
        #[arg(short, long = "script")]
        scripts: Vec<String>,
        /// The trigger on which to run
        ///
        /// Can be either "push" to trigger on every change (default)
        /// or "tag:regex" to trigger on every tag matching a regex.
        #[arg(long = "on", default_value = "push")]
        trigger: Trigger,
        /// Refreshes the repo with this delay
        /// 
        /// Can be a number postfixed with s(econd), m(inutes), h(ours), d(ays)
        #[arg(long = "every", default_value = "1m")]
        delay: DurationString,
        /// Runs an HTTP server on the URL, which allows to trigger by calling it
        #[arg(long)]
        http: Option<String>,
    },
}

/// Track a repository for changes and run scripts when it happens
#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}
