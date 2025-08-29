/// Minimal universal formatter used by some modules

#[derive(Debug, Clone)]
pub struct OutputSection {
    pub title: String,
    pub lines: Vec<String>,
    // Minimal extra fields used by beautiful_ls
    pub content: Option<String>,
    pub collapsible: bool,
    pub collapsed: bool,
}

impl OutputSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
            content: None,
            collapsible: false,
            collapsed: false,
        }
    }

    pub fn add_line(&mut self, line: impl Into<String>) {
        self.lines.push(line.into());
    }
}

// Minimal stubs for types used by beautiful_ls
#[derive(Debug, Clone)]
pub enum CommandOutput {
    Text(String),
    Error {
        message: String,
        details: Option<String>,
        code: Option<i32>,
    },
    KeyValue {
        pairs: Vec<(String, String)>,
        title: Option<String>,
    },
    MultiSection {
        sections: Vec<OutputSection>,
    },
    List {
        items: Vec<FileInfo>,
        title: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub file_type: FileType,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub permissions: Option<String>,
    pub owner: String,
    pub group: String,
}

#[derive(Debug, Clone)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Other,
    SymbolicLink,
    RegularFile,
}

#[derive(Debug, Clone)]
pub struct UniversalFormatter;

impl UniversalFormatter {
    pub fn new() -> Result<Self, String> {
        Ok(UniversalFormatter)
    }
    pub fn format(&self, _out: &CommandOutput) -> Result<String, String> {
        Ok(String::new())
    }
    pub fn format_file_listing(&self, _files: &[FileInfo]) -> Result<String, String> {
        Ok(String::new())
    }
}
