// Simple test for the enhanced ls command functionality
use std::path::Path;

// Mock structures for testing
struct MockRepository;
struct MockStatus;

impl MockRepository {
    fn discover(_path: &Path) -> Option<Self> {
        Some(MockRepository)
    }
    
    fn workdir(&self) -> Option<&Path> {
        Some(Path::new("."))
    }
    
    fn statuses(&self, _opts: Option<&mut MockStatusOptions>) -> Result<MockStatuses, ()> {
        Ok(MockStatuses)
    }
}

struct MockStatusOptions;
impl MockStatusOptions {
    fn new() -> Self { MockStatusOptions }
    fn include_untracked(&mut self, _val: bool) -> &mut Self { self }
    fn include_ignored(&mut self, _val: bool) -> &mut Self { self }
    fn pathspec<P: AsRef<Path>>(&mut self, _path: P) -> &mut Self { self }
}

struct MockStatuses;
impl MockStatuses {
    fn is_empty(&self) -> bool { false }
    fn get(&self, _index: usize) -> Option<MockStatusEntry> {
        Some(MockStatusEntry)
    }
}

struct MockStatusEntry;
impl MockStatusEntry {
    fn status(&self) -> MockStatusFlags {
        MockStatusFlags::WT_MODIFIED
    }
}

#[derive(Clone, Copy)]
enum MockStatusFlags {
    WT_NEW,
    WT_MODIFIED,
    WT_DELETED,
    WT_RENAMED,
    WT_TYPECHANGE,
    INDEX_NEW,
    INDEX_MODIFIED,
    INDEX_DELETED,
    INDEX_RENAMED,
    INDEX_TYPECHANGE,
    IGNORED,
}

impl MockStatusFlags {
    fn contains(self, _other: Self) -> bool {
        true
    }
}

// Mock color enum
#[derive(Clone, Copy)]
enum MockColour {
    Red,
    Yellow,
    Blue,
    Green,
    Cyan,
    Magenta,
    White,
    Fixed(u8),
}

impl MockColour {
    fn paint(self, text: &str) -> String {
        format!("\x1b[{}m{}\x1b[0m", self.ansi_code(), text)
    }
    
    fn ansi_code(self) -> u8 {
        match self {
            MockColour::Red => 31,
            MockColour::Yellow => 33,
            MockColour::Blue => 34,
            MockColour::Green => 32,
            MockColour::Cyan => 36,
            MockColour::Magenta => 35,
            MockColour::White => 37,
            MockColour::Fixed(code) => code,
        }
    }
}

/// Enhanced Git status detection with color information
fn git_status_with_color(repo: &MockRepository, path: &Path) -> (String, Option<MockColour>) {
    // Get relative path from repository root
    let repo_workdir = match repo.workdir() {
        Some(workdir) => workdir,
        None => return (" ".to_string(), None),
    };
    
    let relative_path = match path.strip_prefix(repo_workdir) {
        Ok(rel_path) => rel_path,
        Err(_) => return (" ".to_string(), None),
    };
    
    let mut opts = MockStatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .pathspec(relative_path);
    
    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => statuses,
        Err(_) => return (" ".to_string(), None),
    };
    
    if statuses.is_empty() {
        return (" ".to_string(), None);
    }
    
    // Check the first matching status entry
    let status_entry = match statuses.get(0) {
        Some(entry) => entry,
        None => return (" ".to_string(), None),
    };
    
    let status_flags = status_entry.status();
    
    // Determine status character and color based on Git status flags
    match status_flags {
        // Untracked files
        s if matches!(s, MockStatusFlags::WT_NEW) => ("?".to_string(), Some(MockColour::Red)),
        
        // Modified in working tree
        s if matches!(s, MockStatusFlags::WT_MODIFIED) => ("M".to_string(), Some(MockColour::Yellow)),
        
        // Deleted in working tree
        s if matches!(s, MockStatusFlags::WT_DELETED) => ("D".to_string(), Some(MockColour::Red)),
        
        // Renamed in working tree
        s if matches!(s, MockStatusFlags::WT_RENAMED) => ("R".to_string(), Some(MockColour::Blue)),
        
        // Type changed in working tree
        s if matches!(s, MockStatusFlags::WT_TYPECHANGE) => ("T".to_string(), Some(MockColour::Cyan)),
        
        // Added to index
        s if matches!(s, MockStatusFlags::INDEX_NEW) => ("A".to_string(), Some(MockColour::Green)),
        
        // Modified in index
        s if matches!(s, MockStatusFlags::INDEX_MODIFIED) => ("M".to_string(), Some(MockColour::Green)),
        
        // Deleted in index
        s if matches!(s, MockStatusFlags::INDEX_DELETED) => ("D".to_string(), Some(MockColour::Green)),
        
        // Renamed in index
        s if matches!(s, MockStatusFlags::INDEX_RENAMED) => ("R".to_string(), Some(MockColour::Green)),
        
        // Type changed in index
        s if matches!(s, MockStatusFlags::INDEX_TYPECHANGE) => ("T".to_string(), Some(MockColour::Green)),
        
        // Ignored files
        s if matches!(s, MockStatusFlags::IGNORED) => ("!".to_string(), Some(MockColour::Fixed(8))), // Dark gray
        
        // Clean files (tracked but unchanged)
        _ => (" ".to_string(), None),
    }
}

/// Apply color to filename based on file type and Git status
fn apply_color_to_name(name: &str, is_dir: bool, git_color: Option<MockColour>) -> String {
    let base_color = if is_dir {
        MockColour::Blue
    } else {
        // Determine color based on file extension
        match std::path::Path::new(name).extension().and_then(|s| s.to_str()) {
            Some("rs") => MockColour::Fixed(208), // Orange for Rust files
            Some("md") => MockColour::White,
            Some("toml") | Some("yaml") | Some("yml") | Some("json") => MockColour::Yellow,
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") => MockColour::Magenta,
            Some("zip") | Some("tar") | Some("gz") | Some("bz2") | Some("xz") => MockColour::Red,
            Some("exe") | Some("bin") => MockColour::Green,
            Some("sh") | Some("bash") | Some("zsh") | Some("fish") => MockColour::Green,
            _ => MockColour::White,
        }
    };
    
    // Use Git status color if available, otherwise use file type color
    let final_color = git_color.unwrap_or(base_color);
    final_color.paint(name)
}

/// Apply color to icon based on Git status
fn apply_color_to_icon(icon: &str, git_color: Option<MockColour>) -> String {
    match git_color {
        Some(color) => color.paint(icon),
        None => icon.to_string(),
    }
}

fn icon_for(path: &Path, is_dir: bool) -> &'static str {
    if is_dir {
        return "📁";
    }
    match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "rs" => "🦀",
        "md" => "📄",
        "toml" => "⚙️",
        "png" | "jpg" | "jpeg" | "gif" => "🖼️",
        "zip" | "tar" | "gz" => "📦",
        _ => "📄",
    }
}

fn main() {
    println!("Testing enhanced ls command functionality...");
    
    // Test Git status detection
    let repo = MockRepository::discover(Path::new(".")).unwrap();
    let test_path = Path::new("test.rs");
    let (status, color) = git_status_with_color(&repo, test_path);
    println!("Git status for test.rs: '{}' with color: {:?}", status, color.is_some());
    
    // Test color application
    let colored_name = apply_color_to_name("test.rs", false, color);
    println!("Colored name: {}", colored_name);
    
    // Test icon with color
    let icon = icon_for(test_path, false);
    let colored_icon = apply_color_to_icon(icon, color);
    println!("Colored icon: {}", colored_icon);
    
    // Test different file types
    let test_files = vec![
        ("README.md", false),
        ("Cargo.toml", false),
        ("src", true),
        ("image.png", false),
        ("archive.zip", false),
        ("script.sh", false),
    ];
    
    for (filename, is_dir) in test_files {
        let path = Path::new(filename);
        let icon = icon_for(path, is_dir);
        let (git_status, git_color) = git_status_with_color(&repo, path);
        let colored_name = apply_color_to_name(filename, is_dir, git_color);
        let colored_icon = apply_color_to_icon(icon, git_color);
        
        println!("{} {} [{}]", colored_icon, colored_name, git_status);
    }
    
    println!("✅ Enhanced ls command functionality test completed successfully!");
}