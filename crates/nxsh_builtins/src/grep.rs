use anyhow::Result;
use pcre2::bytes::Regex;
use rayon::prelude::*;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use ansi_term::Colour;

pub struct GrepOptions {
    pub pattern: String,
    pub json: bool,
}

pub fn grep_cli(opts: GrepOptions, paths: &[String]) -> Result<()> {
    let regex = Regex::new(&opts.pattern)?;
    let files: Vec<String> = if paths.is_empty() {
        vec![".".into()]
    } else {
        paths.to_vec()
    };

    files.par_iter().try_for_each(|p| grep_path(&regex, Path::new(p), opts.json))?;
    Ok(())
}

fn grep_path(re: &Regex, path: &Path, json: bool) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            grep_path(re, &entry.path(), json)?;
        }
    } else {
        let mut file = fs::File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        if let Ok(text) = std::str::from_utf8(&buf) {
            for (lineno, line) in text.lines().enumerate() {
                if re.is_match(line.as_bytes())? {
                    if json {
                        println!("{{\"file\":\"{}\",\"line\":{},\"text\":\"{}\"}}",
                            path.display(), lineno+1, line.replace('\"', "\\\""));
                    } else {
                        println!("{}:{}:{}",
                            Colour::Blue.paint(path.display().to_string()),
                            Colour::Green.paint(format!("{}", lineno+1)),
                            line);
                    }
                }
            }
        }
    }
    Ok(())
} 