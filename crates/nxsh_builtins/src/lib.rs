//! NexusShell Builtins Library
//! 
//! Comprehensive collection of built-in commands with Pure Rust implementations.
//! Maintains cross-platform compatibility and enterprise-grade functionality.

use anyhow::Result;
use clap::Parser; // for UpdateArgs::parse_from
#[cfg(feature = "async-runtime")]
use once_cell::sync::Lazy;
#[cfg(feature = "async-runtime")]
static GLOBAL_RT: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build global tokio runtime")
});
use nxsh_core::error::{ShellError, ErrorKind, RuntimeErrorKind};

/// Enum representing all builtin commands
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuiltinCommand {
    Alias,
    History,
    Echo,
    Cd,
    Pwd,
    Ls,
    Cat,
    Cp,
    Mv,
    Rm,
    Mkdir,
    Rmdir,
    Touch,
    Ln,
    Find,
    Grep,
    Sort,
    Uniq,
    Head,
    Tail,
    Wc,
    Cut,
    Awk,
    Sed,
    Tr,
    Fold,
    Ps,
    Top,
    Kill,
    Free,
    Uptime,
    Df,
    Du,
    Mount,
    Chmod,
    Chown,
    Chgrp,
    Date,
    Sleep,
    Yes,
    Test,
    Export,
    Env,
    Which,
    Whereis,
    Whoami,
    Id,
    Groups,
    Su,
    Sudo,
    Tar,
    Gzip,
    Gunzip,
    Zip,
    Unzip,
    Wget,
    Curl,
    Ssh,
    Scp,
    Rsync,
    Ping,
    Netstat,
    Ifconfig,
    Route,
    Iptables,
    Crontab,
    At,
    Nohup,
    Jobs,
    Bg,
    Fg,
    Disown,
    Comm,
    Diff,
    Patch,
    Tee,
    Xargs,
    Csplit,
    Expand,
    Unexpand,
    Pr,
    Nl,
    Od,
    Xxd,
    Hexdump,
    Strings,
    Base64,
    Md5sum,
    Sha1sum,
    Sha256sum,
    Cksum,
}

// Core shell functionality modules
pub mod alias;
pub use alias::alias_cli;

pub mod history;
pub use history::history_cli;

pub mod echo;
pub use echo::echo_cli;

pub mod cd;
pub use cd::cd_cli;

pub mod pwd;
pub use pwd::pwd_cli;

pub mod ls;
pub use ls::ls_cli;

pub mod cat;
pub use cat::cat_cli;

// cp is always available (was previously wrongly cfg gated alongside cal)
pub mod cp;
pub use cp::cp_cli;

pub mod mv;
pub use mv::mv_cli;

pub mod rm;
pub use rm::rm_cli;

pub mod mkdir;
pub use mkdir::mkdir_cli;

pub mod rmdir;
pub use rmdir::rmdir_cli;

pub mod touch;
pub use touch::touch_cli;

pub mod ln;
pub use ln::ln_cli;

// Regex-heavy utilities (gated behind advanced-regex). In super-min or when disabled,
// we expose stubs to keep the command table consistent.
#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub mod find;
#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub use find::find_cli;
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub mod find { use anyhow::{Result, anyhow}; pub fn find_cli(_: &[String]) -> Result<()> { Err(anyhow!("find disabled in this build")) } }
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub use find::find_cli;

#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub mod grep;
#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub use grep::grep_cli;
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub mod grep { use anyhow::{Result, anyhow}; pub fn grep_cli(_: &[String]) -> Result<()> { Err(anyhow!("grep disabled in this build")) } }
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub use grep::grep_cli;

pub mod uniq;
pub use uniq::uniq_cli;

// Job control related new builtins (disown, wait, suspend)
pub mod disown;
pub use disown::disown_cli;
pub mod wait;
pub use wait::wait_cli;
pub mod suspend;
pub use suspend::suspend_cli;

pub mod head;
pub use head::head_cli;

pub mod tail;
pub use tail::tail_cli;

pub mod wc;
pub use wc::wc_cli;

pub mod cut;
pub use cut::cut_cli;

pub mod sort;
pub use sort::sort_cli;

#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub mod awk;
#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub use awk::awk_cli;
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub mod awk { use anyhow::{Result, anyhow};
    // Signature matches real implementation (adds &mut ShellContext)
    pub fn awk_cli(_: &[String], _: &mut nxsh_core::context::ShellContext) -> Result<()> { Err(anyhow!("awk disabled in super-min")) }
}
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub use awk::awk_cli;

#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub mod sed;
#[cfg(all(not(feature = "super-min"), feature = "advanced-regex"))]
pub use sed::sed_cli;
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub mod sed { use anyhow::{Result, anyhow}; pub fn sed_cli(_: &[String]) -> Result<()> { Err(anyhow!("sed disabled in this build")) } }
#[cfg(any(feature = "super-min", not(feature = "advanced-regex")))]
pub use sed::sed_cli;

pub mod tr;
pub use tr::tr_cli;

pub mod fold;
pub use fold::fold_cli;

// System information commands
pub mod ps;
pub use ps::ps_cli;

pub mod top;
pub use top::top_cli;

pub mod kill;
pub use kill::kill_cli;

pub mod free;
pub use free::free_cli;

pub mod uptime;
pub use uptime::uptime_cli;

// User and system utilities
pub mod whoami;
pub use whoami::whoami_cli;

pub mod export_builtin;
pub use export_builtin::export_cli;

#[cfg(feature = "compression-lzma")]
pub mod xz;
#[cfg(feature = "compression-lzma")]
pub use xz::xz_cli;
#[cfg(not(feature = "compression-lzma"))]
pub mod xz { use anyhow::{Result, anyhow}; pub fn xz_cli(_: &[String]) -> Result<()> { Err(anyhow!("xz feature disabled")) } }
#[cfg(not(feature = "compression-lzma"))]
pub use xz::xz_cli;

#[cfg(feature = "compression-lzma")]
pub mod unxz;
#[cfg(feature = "compression-lzma")]
pub use unxz::unxz_cli;
#[cfg(not(feature = "compression-lzma"))]
pub mod unxz { use anyhow::{Result, anyhow}; pub fn unxz_cli(_: &[String]) -> Result<()> { Err(anyhow!("unxz feature disabled")) } }
#[cfg(not(feature = "compression-lzma"))]
pub use unxz::unxz_cli;

// Text processing utilities
pub mod csplit;
pub use csplit::csplit_cli;

pub mod expand;
pub use expand::expand_cli;

pub mod unexpand;
pub use unexpand::unexpand_cli;

pub mod pr;
pub use pr::pr_cli;

pub mod nl;
pub use nl::nl_cli;

// Binary utilities
pub mod od;
pub use od::od_cli;

pub mod xxd;
pub use xxd::xxd_cli;

pub mod hexdump;
pub use hexdump::hexdump_cli;

pub mod strings;
pub use strings::strings_cli;

// Encoding utilities
pub mod base64;
pub use base64::base64_cli;

// Hash utilities
pub mod md5sum;
pub use md5sum::md5sum_cli;

pub mod cksum;
pub use cksum::cksum_cli;

pub mod id;
pub use id::id_cli;

pub mod hostname;
pub use hostname::hostname_cli;

pub mod uname;
pub use uname::uname_cli;

pub mod date;
pub use date::date_cli;
pub mod cal;
pub use cal::cal_cli;

pub mod env;
pub use env::env_cli;

// File permission and ownership
pub mod chmod;
pub use chmod::chmod_cli;

pub mod chown;
pub use chown::chown_cli;

pub mod chgrp;
pub use chgrp::chgrp_cli;

pub mod stat;
pub use stat::stat_cli;

// Archive and compression utilities (Pure Rust implementations)
#[cfg(feature = "compression-tar")]
pub mod tar;
#[cfg(feature = "compression-tar")]
pub use tar::tar_cli;
#[cfg(not(feature = "compression-tar"))]
pub mod tar { use anyhow::{Result, anyhow}; pub fn tar_cli(_: &[String]) -> Result<()> { Err(anyhow!("tar feature disabled")) } }
#[cfg(not(feature = "compression-tar"))]
pub use tar::tar_cli;

#[cfg(feature = "compression-gzip")]
pub mod gzip;
#[cfg(feature = "compression-gzip")]
pub use gzip::{gzip_cli, gunzip_cli, zcat_cli};
#[cfg(not(feature = "compression-gzip"))]
pub mod gzip { use anyhow::{Result, anyhow}; pub fn gzip_cli(_: &[String]) -> Result<()> { Err(anyhow!("gzip feature disabled")) } pub fn gunzip_cli(_: &[String]) -> Result<()> { Err(anyhow!("gunzip feature disabled")) } pub fn zcat_cli(_: &[String]) -> Result<()> { Err(anyhow!("zcat feature disabled")) } }
#[cfg(not(feature = "compression-gzip"))]
pub use gzip::{gzip_cli, gunzip_cli, zcat_cli};

#[cfg(feature = "compression-bzip2")]
pub mod bzip2;
#[cfg(feature = "compression-bzip2")]
pub use bzip2::bzip2_cli;
#[cfg(not(feature = "compression-bzip2"))]
pub mod bzip2 { use anyhow::{Result, anyhow}; pub fn bzip2_cli(_: &[String]) -> Result<()> { Err(anyhow!("bzip2 feature disabled")) } }
#[cfg(not(feature = "compression-bzip2"))]
pub use bzip2::bzip2_cli;

#[cfg(feature = "compression-bzip2")]
pub mod bunzip2;
#[cfg(feature = "compression-bzip2")]
pub use bunzip2::bunzip2_cli;
#[cfg(not(feature = "compression-bzip2"))]
pub mod bunzip2 { use anyhow::{Result, anyhow}; pub fn bunzip2_cli(_: &[String]) -> Result<()> { Err(anyhow!("bunzip2 feature disabled")) } }
#[cfg(not(feature = "compression-bzip2"))]
pub use bunzip2::bunzip2_cli;

#[cfg(feature = "compression-zstd")]
pub mod zstd;
#[cfg(feature = "compression-zstd")]
pub use zstd::zstd_cli;
#[cfg(not(feature = "compression-zstd"))]
pub mod zstd { use anyhow::{Result, anyhow}; pub fn zstd_cli(_: &[String]) -> Result<()> { Err(anyhow!("zstd feature disabled")) } }
#[cfg(not(feature = "compression-zstd"))]
pub use zstd::zstd_cli;

#[cfg(feature = "compression-zstd")]
pub mod unzstd;
#[cfg(feature = "compression-zstd")]
pub use unzstd::unzstd_cli;
#[cfg(not(feature = "compression-zstd"))]
pub mod unzstd { use anyhow::{Result, anyhow}; pub fn unzstd_cli(_: &[String]) -> Result<()> { Err(anyhow!("unzstd feature disabled")) } }
#[cfg(not(feature = "compression-zstd"))]
pub use unzstd::unzstd_cli;

#[cfg(feature = "compression-zip")]
pub mod zip;
#[cfg(feature = "compression-zip")]
pub use zip::zip_cli;
#[cfg(not(feature = "compression-zip"))]
pub mod zip { use anyhow::{Result, anyhow}; pub fn zip_cli(_: &[String]) -> Result<()> { Err(anyhow!("zip feature disabled")) } }
#[cfg(not(feature = "compression-zip"))]
pub use zip::zip_cli;

pub mod sevenz;
pub use sevenz::sevenz_cli;

// select (JMESPath) gated
#[cfg(feature = "json-select")]
pub mod select;
#[cfg(feature = "json-select")]
pub use select::select_cli;
#[cfg(not(feature = "json-select"))]
pub mod select { use anyhow::{Result, anyhow}; pub fn select_cli(_: &[String]) -> Result<()> { Err(anyhow!("select (json-select feature) disabled")) } }
#[cfg(not(feature = "json-select"))]
pub use select::select_cli;

// Network utilities
pub mod curl;
pub use curl::curl_cli;
#[cfg(feature = "dns-tools")]
pub mod dig;
#[cfg(feature = "dns-tools")]
pub use dig::dig_cli;
#[cfg(not(feature = "dns-tools"))]
pub mod dig { use anyhow::{Result, anyhow}; pub fn dig_cli(_: &[String]) -> Result<()> { Err(anyhow!("dig (dns-tools feature) disabled")) } }
#[cfg(not(feature = "dns-tools"))]
pub use dig::dig_cli;

pub mod wget;
pub use wget::wget_cli;

#[cfg(feature = "dns-tools")]
pub mod nslookup;
#[cfg(feature = "dns-tools")]
pub use nslookup::nslookup_cli;
#[cfg(not(feature = "dns-tools"))]
pub mod nslookup { use anyhow::{Result, anyhow}; pub fn nslookup_cli(_: &[String]) -> Result<()> { Err(anyhow!("nslookup (dns-tools feature) disabled")) } }
#[cfg(not(feature = "dns-tools"))]
pub use nslookup::nslookup_cli;
pub mod ping;
pub use ping::ping_cli;

pub mod telnet;
pub use telnet::telnet_cli;

pub mod nc;
pub use nc::nc_cli;

pub mod ssh;
pub use ssh::ssh_cli;

pub mod netstat;
pub use netstat::netstat_cli;

pub mod ss;
pub use ss::ss_cli;

pub mod arp;
pub use arp::arp_cli;

// Text processing and utilities
pub mod diff;
pub use diff::diff_cli;

pub mod patch;
pub use patch::patch_cli;

pub mod comm;
pub use comm::comm_cli;

pub mod join;
pub use join::join_cli;

pub mod paste;
pub use paste::paste_cli;

pub mod split;
pub use split::split_cli;

pub mod fmt;
pub use fmt::fmt_cli;

pub mod sha1sum;
pub use sha1sum::sha1sum_cli;

pub mod sha256sum;
pub use sha256sum::sha256sum_cli;

// System administration
pub mod mount;
pub use mount::mount_cli;

pub mod umount;
pub use umount::umount_cli;

pub mod df;
pub use df::df_cli;

pub mod du;
pub use du::du_cli;

pub mod lsof;
pub use lsof::lsof_cli;

#[cfg(feature = "proc-trace")]
pub mod strace;
#[cfg(feature = "proc-trace")]
pub use strace::strace_cli;
#[cfg(not(feature = "proc-trace"))]
pub mod strace { use anyhow::{Result, anyhow}; pub fn strace_cli(_: &[String]) -> Result<()> { Err(anyhow!("strace feature disabled")) } }
#[cfg(not(feature = "proc-trace"))]
pub use strace::strace_cli;

#[cfg(feature = "proc-trace")]
pub mod ltrace;
#[cfg(feature = "proc-trace")]
pub use ltrace::ltrace_cli;
#[cfg(not(feature = "proc-trace"))]
pub mod ltrace { use anyhow::{Result, anyhow}; pub fn ltrace_cli(_: &[String]) -> Result<()> { Err(anyhow!("ltrace feature disabled")) } }
#[cfg(not(feature = "proc-trace"))]
pub use ltrace::ltrace_cli;

pub mod sudo;
pub use sudo::sudo_cli;

pub mod su;
pub use su::su_cli;

// Process control
pub mod jobs;
pub use jobs::jobs_cli;

pub mod bg;
pub use bg::bg_cli;

pub mod fg;
pub use fg::fg_cli;

pub mod nohup;
pub use nohup::nohup_cli;

pub mod timeout;
pub use timeout::timeout_cli;

pub mod sleep;
pub use sleep::sleep_cli;

#[cfg(feature = "heavy-time")]
pub mod at;
#[cfg(feature = "heavy-time")]
pub use at::at_cli;
#[cfg(not(feature = "heavy-time"))]
pub mod at { use anyhow::{Result, anyhow}; pub fn at_cli(_: &[String]) -> Result<()> { Err(anyhow!("at feature disabled")) } }
#[cfg(not(feature = "heavy-time"))]
pub use at::at_cli;

#[cfg(feature = "heavy-time")]
pub mod crontab;
#[cfg(feature = "heavy-time")]
pub use crontab::crontab_cli;
#[cfg(not(feature = "heavy-time"))]
pub mod crontab { use anyhow::{Result, anyhow}; pub fn crontab_cli(_: &[String]) -> Result<()> { Err(anyhow!("crontab feature disabled")) } }
#[cfg(not(feature = "heavy-time"))]
pub use crontab::crontab_cli;

pub mod nice;
pub use nice::nice_cli;

pub mod renice;
pub use renice::renice_cli;

pub mod ionice;
pub use ionice::ionice_cli;

// System hardware and timing
#[cfg(feature = "heavy-time")]
pub mod hwclock;
#[cfg(feature = "heavy-time")]
pub use hwclock::hwclock_cli;
#[cfg(not(feature = "heavy-time"))]
pub mod hwclock { use anyhow::{Result, anyhow}; pub fn hwclock_cli(_: &[String]) -> Result<()> { Err(anyhow!("hwclock feature disabled")) } }
#[cfg(not(feature = "heavy-time"))]
pub use hwclock::hwclock_cli;

#[cfg(feature = "heavy-time")]
pub mod timedatectl;
#[cfg(feature = "heavy-time")]
pub use timedatectl::timedatectl_cli;
#[cfg(not(feature = "heavy-time"))]
pub mod timedatectl { use anyhow::{Result, anyhow}; pub fn timedatectl_cli(_: &[String]) -> Result<()> { Err(anyhow!("timedatectl feature disabled")) } }
#[cfg(not(feature = "heavy-time"))]
pub use timedatectl::timedatectl_cli;

#[cfg(all(test, feature = "heavy-time"))]
mod timedatectl_tests;

#[cfg(feature = "hardware")]
pub mod lscpu;
#[cfg(feature = "hardware")]
pub use lscpu::lscpu_cli;
#[cfg(not(feature = "hardware"))]
pub mod lscpu { use anyhow::{Result, anyhow}; pub fn lscpu_cli(_: &[String]) -> Result<()> { Err(anyhow!("lscpu feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use lscpu::lscpu_cli;

#[cfg(feature = "hardware")]
pub mod lsblk;
#[cfg(feature = "hardware")]
pub use lsblk::lsblk_cli;
#[cfg(not(feature = "hardware"))]
pub mod lsblk { use anyhow::{Result, anyhow}; pub fn lsblk_cli(_: &[String]) -> Result<()> { Err(anyhow!("lsblk feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use lsblk::lsblk_cli;

#[cfg(feature = "hardware")]
pub mod lspci;
#[cfg(feature = "hardware")]
pub use lspci::lspci_cli;
#[cfg(not(feature = "hardware"))]
pub mod lspci { use anyhow::{Result, anyhow}; pub fn lspci_cli(_: &[String]) -> Result<()> { Err(anyhow!("lspci feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use lspci::lspci_cli;

#[cfg(feature = "hardware")]
pub mod lsusb;
#[cfg(feature = "hardware")]
pub use lsusb::lsusb_cli;
#[cfg(not(feature = "hardware"))]
pub mod lsusb { use anyhow::{Result, anyhow}; pub fn lsusb_cli(_: &[String]) -> Result<()> { Err(anyhow!("lsusb feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use lsusb::lsusb_cli;

#[cfg(feature = "hardware")]
pub mod dmidecode;
#[cfg(feature = "hardware")]
pub use dmidecode::dmidecode_cli;
#[cfg(not(feature = "hardware"))]
pub mod dmidecode { use anyhow::{Result, anyhow}; pub fn dmidecode_cli(_: &[String]) -> Result<()> { Err(anyhow!("dmidecode feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use dmidecode::dmidecode_cli;

#[cfg(feature = "hardware")]
pub mod hdparm;
#[cfg(feature = "hardware")]
pub use hdparm::hdparm_cli;
#[cfg(not(feature = "hardware"))]
pub mod hdparm { use anyhow::{Result, anyhow}; pub fn hdparm_cli(_: &[String]) -> Result<()> { Err(anyhow!("hdparm feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use hdparm::hdparm_cli;

#[cfg(feature = "hardware")]
pub mod smartctl;
#[cfg(feature = "hardware")]
pub use smartctl::smartctl_cli;
#[cfg(not(feature = "hardware"))]
pub mod smartctl { use anyhow::{Result, anyhow}; pub fn smartctl_cli(_: &[String]) -> Result<()> { Err(anyhow!("smartctl feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use smartctl::smartctl_cli;

#[cfg(feature = "hardware")]
pub mod fdisk;
#[cfg(feature = "hardware")]
pub use fdisk::fdisk_cli;
#[cfg(not(feature = "hardware"))]
pub mod fdisk { use anyhow::{Result, anyhow}; pub fn fdisk_cli(_: &[String]) -> Result<()> { Err(anyhow!("fdisk feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use fdisk::fdisk_cli;

#[cfg(feature = "hardware")]
pub mod mkfs;
#[cfg(feature = "hardware")]
pub use mkfs::mkfs_cli;
#[cfg(not(feature = "hardware"))]
pub mod mkfs { use anyhow::{Result, anyhow}; pub fn mkfs_cli(_: &[String]) -> Result<()> { Err(anyhow!("mkfs feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use mkfs::mkfs_cli;

#[cfg(feature = "hardware")]
pub mod fsck;
#[cfg(feature = "hardware")]
pub use fsck::fsck_cli;
#[cfg(not(feature = "hardware"))]
pub mod fsck { use anyhow::{Result, anyhow}; pub fn fsck_cli(_: &[String]) -> Result<()> { Err(anyhow!("fsck feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use fsck::fsck_cli;

#[cfg(feature = "hardware")]
pub mod blkid;
#[cfg(feature = "hardware")]
pub use blkid::blkid_cli;
#[cfg(not(feature = "hardware"))]
pub mod blkid { use anyhow::{Result, anyhow}; pub fn blkid_cli(_: &[String]) -> Result<()> { Err(anyhow!("blkid feature disabled")) } }
#[cfg(not(feature = "hardware"))]
pub use blkid::blkid_cli;

// Shell built-ins
pub mod exit;
pub use exit::exit_cli;

pub mod source;
pub use source::source_cli;

pub mod unset;
pub use unset::unset_cli;

pub mod read_builtin;
pub use read_builtin::read_builtin_cli;

pub mod test_builtin;
pub use test_builtin::test_builtin_cli;

pub mod eval;
pub use eval::eval_cli;

pub mod exec;
pub use exec::exec_cli;

pub mod shift;
pub use shift::shift_cli;

pub mod getopts;
pub use getopts::getopts_cli;

pub mod bind;
pub use bind::bind_cli;

pub mod builtin;
pub use builtin::builtin_cli;

pub mod command;
pub use command::command_cli;

pub mod locate;
pub use locate::locate_cli;

pub mod man;
pub use man::man_cli;

pub mod info;
pub use info::info_cli;

pub mod help;
pub use help::help_cli;

// PowerShell style helper builtins (scaffold)
#[cfg(feature = "powershell-objects")]
pub mod powershell_object;
#[cfg(feature = "powershell-objects")]
pub mod get_command;
#[cfg(feature = "powershell-objects")]
pub use get_command::get_command_cli;
#[cfg(feature = "powershell-objects")]
pub mod get_help;
#[cfg(feature = "powershell-objects")]
pub use get_help::get_help_cli;
#[cfg(not(feature = "powershell-objects"))]
pub mod powershell_object { #[allow(dead_code)] pub fn disabled(){} }
#[cfg(not(feature = "powershell-objects"))]
pub fn get_command_cli(_: &[String]) -> Result<(), ShellError> {
    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "PowerShell objects feature disabled"))
}
#[cfg(not(feature = "powershell-objects"))]
pub fn get_help_cli(_: &[String]) -> Result<(), ShellError> {
    Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "PowerShell objects feature disabled"))
}

// Control flow
pub mod if_cmd;
pub use if_cmd::if_cmd_cli;

pub mod case;
pub use case::case_cli;

pub mod while_cmd;
pub use while_cmd::while_cmd_cli;

pub mod for_cmd;
pub use for_cmd::for_cmd_cli;

pub mod until;
pub use until::until_cli;

// select は json-select feature で上部 Network utilities 手前に gate 済み

pub mod break_builtin;
pub use break_builtin::break_cli;

pub mod continue_builtin;
pub use continue_builtin::continue_builtin_cli;

pub mod return_builtin;
pub use return_builtin::return_builtin_cli;

pub mod function;
pub use function::function_cli;

pub mod local;
pub use local::local_cli;

pub mod readonly;
pub use readonly::readonly_cli;

pub mod declare;
pub use declare::declare_cli;

pub mod vars;
pub use vars::{let_cli, declare_cli as vars_declare_cli, printf_cli};

pub mod typeset;
pub use typeset::typeset_cli;

// Directory stack operations
pub mod pushd;
pub use pushd::pushd_cli;

pub mod popd;
pub use popd::popd_cli;

pub mod dirs;
pub use dirs::dirs_cli;

// I/O redirection helpers
pub mod tee;
pub use tee::tee_cli;

pub mod xargs;
pub use xargs::xargs_cli;

// Advanced utilities
#[cfg(feature = "math-advanced")]
pub mod bc;
#[cfg(feature = "math-advanced")]
pub use bc::bc_cli;
#[cfg(not(feature = "math-advanced"))]
pub mod bc { use anyhow::{Result, anyhow}; pub fn bc_cli(_: &[String]) -> Result<()> { Err(anyhow!("bc feature disabled")) } }
#[cfg(not(feature = "math-advanced"))]
pub use bc::bc_cli;

#[cfg(feature = "math-advanced")]
pub mod dc;
#[cfg(feature = "math-advanced")]
pub use dc::dc_cli;
#[cfg(not(feature = "math-advanced"))]
pub mod dc { use anyhow::{Result, anyhow}; pub fn dc_cli(_: &[String]) -> Result<()> { Err(anyhow!("dc feature disabled")) } }
#[cfg(not(feature = "math-advanced"))]
pub use dc::dc_cli;

#[cfg(feature = "math-advanced")]
pub mod expr;
#[cfg(feature = "math-advanced")]
pub use expr::expr_cli;
#[cfg(not(feature = "math-advanced"))]
pub mod expr { use anyhow::{Result, anyhow}; pub fn expr_cli(_: &[String]) -> Result<()> { Err(anyhow!("expr feature disabled")) } }
#[cfg(not(feature = "math-advanced"))]
pub use expr::expr_cli;

pub mod seq;
pub use seq::seq_cli;

pub mod yes;
pub use yes::yes_cli;

pub mod true_cmd;
pub use true_cmd::true_cmd_cli;

pub mod false_cmd;
pub use false_cmd::false_cmd_cli;

pub mod time_cmd;
pub use time_cmd::time_cli;

// Signal handling
pub mod trap;
pub use trap::trap_cli;

pub mod signal;
pub use signal::signal_cli;

// Resource limits
pub mod ulimit;
pub use ulimit::ulimit_cli;

pub mod umask;
pub use umask::umask_cli;

// System maintenance and updates (requires async runtime; heavy networking)
#[cfg(feature = "async-runtime")]
pub mod update;
#[cfg(feature = "async-runtime")]
pub use update::update_cli;
#[cfg(not(feature = "async-runtime"))]
pub mod update { use anyhow::{Result, anyhow}; pub fn update_cli(_: &[String]) -> Result<()> { Err(anyhow!("update: disabled in this build")) } }
#[cfg(not(feature = "async-runtime"))]
pub use update::update_cli;

#[cfg(feature = "package-management")]
pub mod package;
#[cfg(feature = "package-management")]
pub use package::package_cli;
#[cfg(not(feature = "package-management"))]
pub mod package { use anyhow::{Result, anyhow}; pub fn package_cli(_: &[String]) -> Result<()> { Err(anyhow!("package-management feature disabled")) } }
#[cfg(not(feature = "package-management"))]
pub use package::package_cli;

// Extended grep utilities (regex heavy) – stub in super-min
#[cfg(not(feature = "super-min"))]
pub mod egrep;
#[cfg(not(feature = "super-min"))]
pub use egrep::egrep_cli;
#[cfg(feature = "super-min")]
pub mod egrep { use anyhow::{Result, anyhow}; pub fn egrep_cli(_: &[String]) -> Result<()> { Err(anyhow!("egrep disabled in super-min")) } }
#[cfg(feature = "super-min")]
pub use egrep::egrep_cli;

#[cfg(not(feature = "super-min"))]
pub mod fgrep;
#[cfg(not(feature = "super-min"))]
pub use fgrep::fgrep_cli;
#[cfg(feature = "super-min")]
pub mod fgrep { use anyhow::{Result, anyhow}; pub fn fgrep_cli(_: &[String]) -> Result<()> { Err(anyhow!("fgrep disabled in super-min")) } }
#[cfg(feature = "super-min")]
pub use fgrep::fgrep_cli;

// Advanced text processing
pub mod group_by;
pub use group_by::group_by_cli;

// Utility modules
pub mod common;
pub use common::*;

// Logging statistics builtin (exposes runtime logging metrics)
#[cfg(feature = "logging")]
pub mod logstats;
#[cfg(feature = "logging")]
pub use logstats::logstats_cli;
#[cfg(not(feature = "logging"))]
pub mod logstats_builtin;
#[cfg(not(feature = "logging"))]
pub use logstats_builtin::logstats_cli;

// Error handling and result types
pub type BuiltinResult<T> = Result<T, ShellError>;

// ---------------------------------------------------------------------------
// Introspection helpers (used by BusyBox mode and external embedding)
// Auto-generated list (see build.rs) to avoid drift with execute_builtin dispatch.
// ---------------------------------------------------------------------------
// The build script emits builtins_generated.rs into OUT_DIR.
include!(concat!(env!("OUT_DIR"), "/builtins_generated.rs"));

/// Return true if name corresponds to a builtin we can dispatch (generated).
pub fn is_builtin_name(name: &str) -> bool { is_builtin_name_generated(name) }

/// List all builtin names (generated, alphabetical order).
pub fn list_builtin_names() -> Vec<&'static str> { list_builtin_names_generated() }

/// Entry point for builtin command execution
pub fn execute_builtin(command: &str, args: &[String]) -> BuiltinResult<()> {
    // In minimal builds (`nxsh_core` without the `error-rich` feature) there is
    // no `impl From<anyhow::Error> for ShellError>`; avoid using `map_err(ShellError::from)`.
    // Provide a small local adapter so we keep code size low and compile without that feature.
    #[allow(clippy::needless_pass_by_value)]
    fn shellerr_from_anyhow(err: anyhow::Error) -> ShellError {
        // If the rich conversion exists (feature enabled) we delegate to it.
        use nxsh_core::error::RuntimeErrorKind;
        use nxsh_core::ErrorKind;
        ShellError::new(
            ErrorKind::RuntimeError(RuntimeErrorKind::ConversionError),
            err.to_string(),
        )
    }
    match command {
        "alias" => alias_cli(args).map_err(shellerr_from_anyhow),
        "cd" => {
            let mut ctx = nxsh_core::context::ShellContext::new();
            cd_cli(args, &mut ctx)},
        "pwd" => {
            let ctx = nxsh_core::context::ShellContext::new();
            pwd_cli(args, &ctx).map_err(shellerr_from_anyhow)
        },
        "ls" => ls_cli(args).map_err(shellerr_from_anyhow),
        "cat" => cat_cli(args).map_err(shellerr_from_anyhow),
        "cp" => {
            #[cfg(all(feature = "async-runtime", not(feature = "super-min")))]
            { return GLOBAL_RT.block_on(async { cp_cli(args).await }).map_err(shellerr_from_anyhow); }
            #[cfg(feature = "super-min")]
            { return cp_cli(args).map_err(shellerr_from_anyhow); }
            #[cfg(all(not(feature = "async-runtime"), not(feature = "super-min")))]
            { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "cp: async runtime not available (enable feature 'async-runtime' or 'super-min')")); }
        },
        "mv" => mv_cli(args).map_err(shellerr_from_anyhow),
        "rm" => rm_cli(args).map_err(shellerr_from_anyhow),
        "mkdir" => mkdir_cli(args).map_err(shellerr_from_anyhow),
        "rmdir" => rmdir_cli(args).map_err(shellerr_from_anyhow),
        "touch" => touch_cli(args).map_err(shellerr_from_anyhow),
        "ln" => ln_cli(args).map_err(shellerr_from_anyhow),
        "find" => find_cli(args).map_err(shellerr_from_anyhow),
        "grep" => grep_cli(args).map_err(shellerr_from_anyhow),
        "sort" => sort_cli(args).map_err(shellerr_from_anyhow),
        "uniq" => uniq_cli(args).map_err(shellerr_from_anyhow),
        "head" => head_cli(args).map_err(shellerr_from_anyhow),
        "tail" => tail_cli(args).map_err(shellerr_from_anyhow),
        "wc" => wc_cli(args).map_err(shellerr_from_anyhow),
        "cut" => cut_cli(args).map_err(shellerr_from_anyhow),
        "awk" => {
            let mut ctx = nxsh_core::context::ShellContext::new();
            awk_cli(args, &mut ctx).map_err(shellerr_from_anyhow)
        },
        "sed" => sed_cli(args).map_err(shellerr_from_anyhow),
    #[cfg(feature = "logging")]
    "logstats" => logstats_cli(args).map_err(shellerr_from_anyhow),
        "tr" => tr_cli(args).map_err(shellerr_from_anyhow),
        "fold" => fold_cli(args).map_err(shellerr_from_anyhow),
        "ps" => ps_cli(args).map_err(shellerr_from_anyhow),
        "top" => top_cli(args).map_err(shellerr_from_anyhow),
        "kill" => kill_cli(args).map_err(shellerr_from_anyhow),
        "free" => free_cli(args).map_err(shellerr_from_anyhow),
        "uptime" => uptime_cli(args).map_err(shellerr_from_anyhow),
        "id" => id_cli(args).map_err(shellerr_from_anyhow),
        "whoami" => whoami_cli(args).map_err(shellerr_from_anyhow),
        "hostname" => hostname_cli(args).map_err(shellerr_from_anyhow),
        "uname" => uname_cli(args).map_err(shellerr_from_anyhow),
        "date" => {
            #[cfg(all(feature = "async-runtime", not(feature = "super-min")))]
            { return GLOBAL_RT.block_on(async { date_cli(args).await }).map_err(shellerr_from_anyhow); }
            #[cfg(feature = "super-min")]
            { return futures::executor::block_on(date_cli(args)).map_err(shellerr_from_anyhow); }
            #[cfg(all(not(feature = "async-runtime"), not(feature = "super-min")))]
            { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "date: async runtime not available in this build")); }
        },
        "cal" => {
            #[cfg(all(feature = "async-runtime", not(feature = "super-min")))]
            { return GLOBAL_RT.block_on(async { cal_cli(args.to_vec()).await.map(|_| ()) }); }
            #[cfg(feature = "super-min")]
            { let _ = futures::executor::block_on(cal_cli(args.to_vec())); return Ok(()); }
            #[cfg(all(not(feature = "async-runtime"), not(feature = "super-min")))]
            { return Err(ShellError::new(ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument), "cal: async runtime not available in this build")); }
        },
        "env" => env_cli(args).map_err(shellerr_from_anyhow),
                "chmod" => chmod_cli(args).map_err(shellerr_from_anyhow),
        "chown" => chown_cli(args).map_err(shellerr_from_anyhow),
        "chgrp" => chgrp_cli(args).map_err(shellerr_from_anyhow),
        "stat" => stat_cli(args).map_err(shellerr_from_anyhow),
        "tar" => tar_cli(args).map_err(shellerr_from_anyhow),
        "gzip" => gzip_cli(args).map_err(shellerr_from_anyhow),
        "bzip2" => bzip2_cli(args).map_err(shellerr_from_anyhow),
        "bunzip2" => bunzip2_cli(args).map_err(shellerr_from_anyhow),
        "xz" => xz_cli(args).map_err(shellerr_from_anyhow),
        "zstd" => zstd_cli(args).map_err(shellerr_from_anyhow),
        "unzstd" => unzstd_cli(args).map_err(shellerr_from_anyhow),
        "zip" => zip_cli(args).map_err(shellerr_from_anyhow),
        "7z" => sevenz_cli(args).map_err(shellerr_from_anyhow),
    "disown" => disown_cli(args).map_err(shellerr_from_anyhow),
    "suspend" => suspend_cli(args).map_err(shellerr_from_anyhow),
    "wait" => wait_cli(args).map_err(shellerr_from_anyhow),
        "printf" => {
            // Provided through vars::printf_cli (format subset). Re-route.
            printf_cli(args).map_err(shellerr_from_anyhow)
        },
    #[cfg(feature = "async-runtime")]
    "update" => {
        // update_cli expects a parsed UpdateArgs struct; perform minimal parsing here.
        // Fallback: if parsing fails, surface as ConversionError.
        use crate::update::UpdateArgs;
        match UpdateArgs::try_parse_from(std::iter::once("update").chain(args.iter().map(|s| s.as_str()))) {
            Ok(uargs) => {
                GLOBAL_RT.block_on(update_cli(uargs)).map_err(shellerr_from_anyhow)
            }
            Err(e) => Err(ShellError::new(
                ErrorKind::RuntimeError(RuntimeErrorKind::InvalidArgument),
                format!("update: invalid arguments: {e}"),
            )),
        }
    },
    #[cfg(feature = "powershell-objects")]
    "Get-Command" | "get-command" => get_command_cli(args).map_err(shellerr_from_anyhow),
    #[cfg(feature = "powershell-objects")]
    "Get-Help" | "get-help" => get_help_cli(args).map_err(shellerr_from_anyhow),
        _ => Err(ShellError::command_not_found(command)),
    }
}
