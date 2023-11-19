#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_qualifications,
    unstable_features
)]

use clap::Parser;

use clap_verbosity_flag::Verbosity;
//use clap_verbosity_flag::{, Verbosity};
use log::LevelFilter;
use serde::Deserialize;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

extern crate g_k_crates_io_client as crates_io;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate threadpool;
#[macro_use]
extern crate log;
extern crate simplelog;

mod model;
mod process;
use crate::process::parse_directory;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets the input directory to parse
    #[arg(short, long)]
    input: Option<String>,

    /// Sets the output directory to store metadata
    #[arg(short, long)]
    output: Option<String>,

    /// Path to log file
    #[arg(short, long)]
    logfile: Option<String>,

    /// Path to optional configuration file
    #[arg(short, long, default_value_t = String::from("./crate-info-mirroring.toml"))]
    config: String,

    /// Number of fetcher processes
    #[arg(short, long)]
    count: Option<u8>,

    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Deserialize)]
struct TomlConfig {
    /// Sets the input directory to parse
    input: String,

    /// Sets the output directory to store metadata
    output: String,

    /// Path to log file
    logfile: Option<String>,

    /// Number of fetcher processes
    count: u8,

    /// logging
    verbose: LevelFilter,
}

struct MainConfig {
    /// Sets the input directory to parse
    input: String,

    /// Sets the output directory to store metadata
    output: String,

    /// Path to log file
    logfile: Option<String>,

    /// Number of fetcher processes
    count: u8,

    /// logging
    verbose: LevelFilter,
}

impl From<Args> for MainConfig {
    fn from(args: Args) -> Self {
        let input = args.input.unwrap_or_else(|| panic!("Input not set"));
        let output = args.output.unwrap_or_else(|| panic!("Output not set"));
        let logfile = args.logfile;
        let count = args.count.unwrap_or(16);
        let verbose = args.verbose.log_level_filter();

        MainConfig {
            input,
            output,
            logfile,
            count,
            verbose,
        }
    }
}

fn merge_configs(args: Args, toml_config: TomlConfig) -> MainConfig {
    let input = args
        .input
        .clone()
        .unwrap_or_else(|| toml_config.input.clone());
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| toml_config.output.clone());
    let logfile = args.logfile.clone().or_else(|| toml_config.logfile.clone());
    let count = args.count.unwrap_or(toml_config.count);
    let arg_loglevel = args.verbose.log_level_filter();
    let verbose = if arg_loglevel > toml_config.verbose {
        arg_loglevel
    } else {
        toml_config.verbose
    };

    MainConfig {
        input,
        output,
        logfile,
        count,
        verbose,
    }
}

fn main() {
    let args = Args::parse();
    let filename = args.config.clone();

    let content = std::fs::read_to_string(&filename);
    let config = if let Ok(content) = content {
        merge_configs(
            args,
            toml::from_str(&content)
                .unwrap_or_else(|err| panic!("Can't parse config file {filename} : {err}")),
        )
    } else {
        MainConfig::from(args)
    };

    // init logs
    if let Some(filename) = config.logfile.as_ref() {
        let logfile = std::fs::File::create(filename).expect("Error: cannot create log file");
        let logger = WriteLogger::new(config.verbose, Config::default(), logfile);
        CombinedLogger::init(vec![logger]).unwrap();
    } else {
        let logger = TermLogger::new(
            config.verbose,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        );
        CombinedLogger::init(vec![logger]).unwrap();
    };

    // check directories
    std::fs::metadata(&config.input)
        .unwrap_or_else(|_| panic!("Can't access to directory {}", &config.input));
    std::fs::metadata(&config.output)
        .unwrap_or_else(|_| panic!("Can't access to directory {}", &config.output));

    parse_directory(config.input, config.output, config.count)
        .expect("Error while processing directories");
}
