//! Minimal PowerShell-like typed object abstraction (scaffold).
//! BusyBox / minimal ビルドでは未使用。`powershell-objects` feature 導入予定の先行土台。
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum PowerShellObject {
    String(String),
    Number(i64),
    Bool(bool),
    List(Vec<PowerShellObject>),
    Map(Vec<(String, PowerShellObject)>),
}

impl From<&str> for PowerShellObject { fn from(s: &str) -> Self { Self::String(s.to_string()) } }
impl From<String> for PowerShellObject { fn from(s: String) -> Self { Self::String(s) } }
impl From<i64> for PowerShellObject { fn from(n: i64) -> Self { Self::Number(n) } }
impl From<bool> for PowerShellObject { fn from(b: bool) -> Self { Self::Bool(b) } }

pub fn emit(objects: &[PowerShellObject]) {
    if std::env::var("NXSH_PS_PIPE").ok().as_deref() == Some("1") {
        for o in objects { println!("{}", serde_json::to_string(o).unwrap()); }
    } else {
        for o in objects { println!("{:?}", o); }
    }
}
