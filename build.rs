use std::process::Command;

fn main() {
    // Re-run if git state changes.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");

    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    // Windows' date command does not support the Unix flags.
    let mut date_command = if cfg!(windows) {
        let mut command = Command::new("powershell.exe");
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[DateTime]::UtcNow.ToString('yyyy-MM-dd')",
        ]);
        command
    } else {
        let mut command = Command::new("date");
        command.args(["-u", "+%Y-%m-%d"]);
        command
    };
    let date = date_command
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=GIT_SHA={sha}");
    println!("cargo:rustc-env=BUILD_DATE={date}");
}
