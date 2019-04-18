#![warn(missing_docs)]

//! A flexible and easy-to-use logger that writes logs to stderr and/or to files
//! or other output streams.
//!
//! To read the log specification from an environment variable and get the log written to `stderr`,
//! start `flexi_logger` e.g. like this:
//! ```rust
//! flexi_logger::Logger::with_env().start().unwrap();
//! ```
//!
//! See
//!
//! * [Logger](struct.Logger.html) for a full description of all configuration options,
//! * and the [writers](writers/index.html) module for the usage of additional log writers,
//! * and [the homepage](https://crates.io/crates/flexi_logger) for how to get started.
//!
//! There are configuration options to e.g.
//!
//! * decide whether you want to write your logs to stderr or to a file,
//! * configure the path and the filenames of the log files,
//! * use file rotation,
//! * specify the line format for the log lines,
//! * define additional log output streams, e.g for alert or security messages,
//! * support changing the log specification while the program is running,
//!
//! `flexi_logger` uses a similar syntax as [`env_logger`](http://crates.io/crates/env_logger/)
//! for specifying which logs should really be written (but is more graceful with the syntax,
//! and can provide error information).

mod flexi_error;
mod flexi_logger;
mod formats;
mod log_specification;
mod logger;
mod primary_writer;
mod reconfiguration_handle;

pub mod writers;

/// Re-exports from log crate
pub use log::{Level, LevelFilter, Record};

pub use crate::flexi_error::FlexiLoggerError;
pub use crate::formats::*;
pub use crate::log_specification::{LogSpecBuilder, LogSpecification};
pub use crate::logger::{Age, Cleanup, Criterion, Duplicate, Logger, Naming};
pub use crate::reconfiguration_handle::ReconfigurationHandle;

use std::io;

/// Function type for Format functions.
pub type FormatFunction = fn(&mut io::Write, &Record) -> Result<(), io::Error>;
