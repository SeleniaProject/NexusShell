use nxsh_core::ShellResult;

pub fn sed(pattern: &str, replacement: &str, input: &str) -> ShellResult<String> {
    // 基本的なsed操作をシミュレート
    Ok(input.replace(pattern, replacement))
}
