use clap::{Arg, Command};
use std::path::PathBuf;

use crate::i18n;

pub struct Args {
    pub json: bool,
    pub scan_only: bool,
    pub verbose: bool,
    pub config: Option<PathBuf>,
    pub select: Option<usize>,
    pub no_pause: bool,
}

pub fn parse_args() -> Args {
    let m = i18n::messages();
    let matches = Command::new("java-runner")
        .version(env!("CARGO_PKG_VERSION"))
        .about(m.about)
        .arg(
            Arg::new("json")
                .long("json")
                .help(m.help_json)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("scan-only")
                .long("scan-only")
                .short('s')
                .help(m.help_scan_only)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help(m.help_verbose)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help(m.help_config)
                .value_name("FILE"),
        )
        .arg(
            Arg::new("select")
                .long("select")
                .help(m.help_select)
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("no-pause")
                .long("no-pause")
                .help(m.help_no_pause)
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    Args {
        json: matches.get_flag("json"),
        scan_only: matches.get_flag("scan-only"),
        verbose: matches.get_flag("verbose"),
        config: matches.get_one::<String>("config").map(PathBuf::from),
        select: matches.get_one::<usize>("select").copied(),
        no_pause: matches.get_flag("no-pause"),
    }
}
