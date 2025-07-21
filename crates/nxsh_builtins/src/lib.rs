//! Collection of built-in commands re-exported for convenient linking.

pub mod jobs;

pub use jobs::{fg, bg, jobs_cli as jobs, wait_cli as wait, disown_cli as disown};

pub mod common;

pub use common::logging;

pub mod cd;

pub use cd::cd;

pub mod history;

pub use history::{history_cli as history};

pub mod help;

pub use help::help_cli as help;

pub mod alias;

pub use alias::alias_cli as alias;

pub mod export;

pub use export::export_cli as export;

pub mod set;

pub use set::set_cli as set;

pub mod icons;
pub mod ls;

pub use ls::ls_async as ls;

pub mod grep;

pub use grep::grep_cli as grep;

pub mod tar;

pub use tar::tar_cli as tar;

pub mod select;

pub use select::select_cli as select;

pub mod group_by;

pub use group_by::group_by_cli as group_by;

pub mod vars;

pub use vars::{let_cli as builtin_let, declare_cli as declare, printf_cli as printf}; 