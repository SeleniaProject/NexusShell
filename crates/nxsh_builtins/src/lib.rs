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

pub mod bind;

pub use bind::bind_cli as bind; 

pub mod break_builtin as builtin_break;

pub use builtin_break::break_cli as break_cmd; 

pub mod builtin;

pub use builtin::builtin_cli as builtin_builtin; 

pub mod command;

pub use command::command_cli as command_builtin; 

pub mod complete;

pub use complete::complete_cli as complete_builtin; 

pub mod r#continue;

pub use r#continue::continue_cli as continue_cmd; 

pub mod dirs;

pub use dirs::dirs_cli as dirs_builtin; 

pub mod echo;

pub use echo::echo_cli as echo_builtin; 

pub mod eval;
pub mod exec;
pub mod r#exit;

pub use eval::eval_cli as eval_builtin;
pub use exec::exec_cli as exec_builtin;
pub use r#exit::exit_cli as exit_builtin; 

pub mod getopts;

pub use getopts::getopts_cli as getopts_builtin; 

pub mod hash;

pub use hash::hash_cli as hash_builtin; 

pub mod local;

pub use local::local_cli as local_builtin; 

pub mod pushd;
pub mod popd;

pub use pushd::pushd_cli as pushd_builtin;
pub use popd::popd_cli as popd_builtin; 

pub mod pwd;

pub use pwd::pwd_cli as pwd_builtin; 

pub mod read;

pub use read::read_cli as read_builtin; 

pub mod readonly;

pub use readonly::readonly_cli as readonly_builtin; 

pub mod r#return;

pub use r#return::return_cli as return_builtin; 

pub mod shift;

pub use shift::shift_cli as shift_builtin; 

pub mod source;

pub use source::source_cli as source_builtin; 

pub mod suspend;

pub use suspend::suspend_cli as suspend_builtin; 

pub mod times;

pub use times::times_cli as times_builtin; 

pub mod trap;

pub use trap::trap_cli as trap_builtin; 

pub mod r#type;

pub use r#type::type_cli as type_builtin; 

pub mod ulimit;

pub use ulimit::ulimit_cli as ulimit_builtin; 

pub mod umask;

pub use umask::umask_cli as umask_builtin; 

pub mod unalias;
pub mod unset;

pub use unalias::unalias_cli as unalias_builtin;
pub use unset::unset_cli as unset_builtin; 

pub mod cp;

pub use cp::cp_cli as cp_async; 

pub mod mv;

pub use mv::mv_cli as mv_async; 

pub mod rm;

pub use rm::rm_cli as rm_async; 

pub mod mkdir;

pub use mkdir::mkdir_cli as mkdir_async; 

pub mod rmdir;

pub use rmdir::rmdir_cli as rmdir_async; 

pub mod ln;

pub use ln::ln_cli as ln_async; 

pub mod stat;

pub use stat::stat_cli as stat_async; 

pub mod touch;

pub use touch::touch_cli as touch_async; 

pub mod tree;

pub use tree::tree_cli as tree_async; 

pub mod du;

pub use du::du_cli as du_async; 

pub mod df;

pub use df::df_cli as df_async; 

pub mod sync_cmd;

pub use sync_cmd::sync_cli as sync_async; 

pub mod mount;
pub mod umount_cmd;

pub use mount::mount_cli as mount_async;
pub use umount_cmd::umount_cli as umount_async; 

pub mod shred;

pub use shred::shred_cli as shred_async; 

pub mod split;

pub use split::split_cli as split_async; 

pub mod cat;

pub use cat::cat_cli as cat_async; 

pub mod more;

pub use more::more_cli as more_async; 

pub mod less;

pub use less::less_cli as less_async; 

pub mod awk;

pub use awk::awk_cli as awk_builtin; 
 
pub mod egrep;

pub use sed::sed_cli as sed_builtin; 
 
pub mod tr;

pub use tr::tr_cli as tr_builtin; 
 
pub mod cut;

pub use cut::cut_cli as cut_builtin; 
 
pub mod paste;

pub use paste::paste_cli as paste_builtin; 
 
pub mod sort;

pub use sort::sort_cli as sort_builtin; 
 
pub mod uniq;

pub use uniq::uniq_cli as uniq_builtin; 
 
pub mod head;

pub use head::head_cli as head_builtin; 
 
pub mod tail;

pub use tail::tail_cli as tail_builtin; 
 
pub mod wc;

pub use wc::wc_cli as wc_builtin; 
 
pub mod fmt;

pub use fmt::fmt_cli as fmt_builtin; 
 
pub mod fold;

pub use fold::fold_cli as fold_builtin; 
 
pub mod join;

pub use join::join_cli as join_builtin; 
 
pub mod comm;

pub use comm::comm_cli as comm_builtin; 
 
pub mod diff;

pub use diff::diff_cli as diff_builtin; 
 
pub mod patch;
pub use patch::patch_cli as patch_builtin; 

pub mod rev;
pub use rev::rev_cli as rev_builtin; 
 
pub mod ps;
pub use ps::ps_cli as ps_builtin; 
 
pub mod top;
pub use top::top_cli as top_builtin; 
 
pub mod htop;
pub use htop::htop_cli as htop_builtin; 
 
pub mod kill;
pub use kill::kill_cli as kill_builtin; 
 
pub mod pkill;
pub use pkill::pkill_cli as pkill_builtin; 

pub mod pgrep;
pub use pgrep::pgrep_cli as pgrep_builtin; 
 
pub mod nice;
pub use nice::nice_cli as nice_builtin; 

pub mod renice;
pub use renice::renice_cli as renice_builtin; 
 
pub mod uptime;
pub use uptime::uptime_cli as uptime_builtin; 

pub mod free;
pub use free::free_cli as free_builtin; 
 
pub mod vmstat;
pub use vmstat::vmstat_cli as vmstat_builtin; 
 
pub mod lsof;
pub use lsof::lsof_cli as lsof_builtin;

pub mod uname;
pub use uname::uname_cli as uname_builtin;
 
pub mod hostname;
pub use hostname::hostname_cli as hostname_builtin;

pub mod env;
pub use env::env_cli as env_builtin;
 
pub mod printenv;
pub use printenv::printenv_cli as printenv_builtin;

pub mod id;
pub use id::id_cli as id_builtin;
 
pub mod groups;
pub use groups::groups_cli as groups_builtin;

pub mod who;
pub use who::who_cli as who_builtin;
 
pub mod time_cmd;
pub use time_cmd::time_cli as time_builtin;
 
// Network tools
pub mod ping;
pub use ping::ping_cli as ping_builtin;

pub mod traceroute;
pub use traceroute::traceroute_cli as traceroute_builtin;

pub mod nslookup;
pub use nslookup::nslookup_cli as nslookup_builtin;

pub mod dig;
pub use dig::dig_cli as dig_builtin;
 
pub mod curl;
pub use curl::curl_cli as curl_builtin; 

pub mod wget;
pub use wget::wget_cli as wget_builtin;

pub mod ssh;
pub use ssh::ssh_cli as ssh_builtin;

pub use egrep::egrep_cli as egrep_sync; 

pub mod fgrep;

pub use fgrep::fgrep_cli as fgrep_sync; 