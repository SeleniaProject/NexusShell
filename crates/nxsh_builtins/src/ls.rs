use anyhow::Result;
use git2::Repository;
use humansize::{file_size_opts as options, FileSize};
use tabled::{Table, Tabled};
use tokio::fs;
use tokio::task;
use crate::icons::icon_for;
use ansi_term::Colour;
use std::path::{PathBuf, Path};

#[derive(Tabled)]
struct LsRow {
    icon: &'static str,
    name: String,
    size: String,
    git: String,
}

pub async fn ls_async(dir: Option<&str>) -> Result<()> {
    let path = PathBuf::from(dir.unwrap_or("."));
    let repo = Repository::discover(&path).ok();

    let mut entries = fs::read_dir(&path).await?;
    let mut rows = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let md = entry.metadata().await?;
        let is_dir = md.is_dir();
        let icon = icon_for(&path, is_dir);
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let size = if is_dir {
            "-".into()
        } else {
            md.len().file_size(options::CONVENTIONAL)?
        };
        let git_status = if let Some(r) = &repo {
            git_status_char(r, &path).unwrap_or(" ")
        } else {
            " "
        };
        rows.push(LsRow { icon, name, size, git: git_status.into() });
    }
    println!("{}", Table::new(rows).to_string());
    Ok(())
}

fn git_status_char(repo: &Repository, path: &Path) -> Option<&'static str> {
    use git2::StatusOptions;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true).pathspec(path);
    let statuses = repo.statuses(Some(&mut opts)).ok()?;
    if statuses.is_empty() {
        return Some(" ");
    }
    let s = statuses.get(0)?;
    let st = s.status();
    if st.is_wt_new() { Some("?") }
    else if st.is_wt_modified() { Some("M") }
    else if st.is_index_new() || st.is_index_modified() { Some("A") }
    else { Some(" ") }
} 