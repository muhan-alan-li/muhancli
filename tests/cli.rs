use std::process::Command;

fn muhan(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_muhan"))
        .args(args)
        .output()
        .expect("muhan must run")
}

#[test]
fn commands_work_without_external_data_files() {
    for args in [
        vec![],
        vec!["about"],
        vec!["experience"],
        vec!["experience", "apera-ai"],
        vec!["projects"],
        vec!["projects", "--tag", "react"],
        vec!["projects", "dune-imperium"],
        vec!["skills", "--group", "Languages"],
        vec!["contact"],
        vec!["version"],
        vec!["--help"],
        vec!["--version"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_muhan"))
            .args(&args)
            .current_dir(std::env::temp_dir())
            .output()
            .unwrap();
        assert!(result.status.success(), "{args:?}: {result:?}");
        assert!(!result.stdout.is_empty(), "{args:?}");
        assert!(
            !result.stdout.contains(&0x1b),
            "piped output has ANSI escapes"
        );
    }
}

#[test]
fn json_is_parseable_and_bad_input_has_a_failure_status() {
    let result = muhan(&["--json", "about"]);
    assert!(result.status.success());
    let profile: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(profile["name"].as_str().is_some());
    let default_profile = muhan(&["--json"]);
    assert!(default_profile.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&default_profile.stdout).unwrap(),
        profile
    );
    let result = muhan(&["projects", "does-not-exist"]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown project"));
}

#[test]
fn completion_scripts_are_available_for_supported_shells() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let result = muhan(&["completions", shell]);
        assert!(result.status.success(), "{shell}: {result:?}");
        let script = String::from_utf8(result.stdout).unwrap();
        assert!(script.contains("MUHAN_COMPLETE"), "{shell}: {script}");
        assert!(!script.contains("Explore:"));
    }
    assert_eq!(
        muhan(&["completions", "unsupported"]).status.code(),
        Some(2)
    );
}

#[test]
fn completion_requests_return_contextual_portfolio_values() {
    for shell in ["bash", "zsh", "fish", "powershell"] {
        for (words, expected) in [
            (vec!["muhan", "exp"], "experience"),
            (vec!["muhan", "experience", "ap"], "apera-ai"),
            (vec!["muhan", "projects", "dune"], "dune-imperium"),
            (vec!["muhan", "projects", "--tag", "Re"], "React"),
            (vec!["muhan", "skills", "--group", "Lang"], "Languages"),
            (vec!["muhan", "--plain", "experience", "nr"], "nrcan"),
            (vec!["muhan", "projects", "--j"], "--json"),
        ] {
            let result = Command::new(env!("CARGO_BIN_EXE_muhan"))
                .env("MUHAN_COMPLETE", shell)
                .env("_CLAP_COMPLETE_INDEX", (words.len() - 1).to_string())
                .args(["--"])
                .args(&words)
                .output()
                .unwrap();
            assert!(result.status.success(), "{shell} {words:?}: {result:?}");
            let candidates = String::from_utf8(result.stdout).unwrap();
            assert!(
                candidates.contains(expected),
                "{shell} {words:?}: expected {expected} in {candidates:?}"
            );
            assert!(!candidates.contains("Explore:"));
        }
    }
    // Completion suggestions must not restrict existing case-insensitive IDs
    // or free-form tag searches in normal invocations.
    assert!(muhan(&["experience", "APERA-AI"]).status.success());
    assert!(muhan(&["projects", "--tag", "arbitrary search"])
        .status
        .success());
}

#[cfg(unix)]
#[test]
fn bash_completion_adapter_can_be_loaded_and_invoked() {
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_muhan"));
    let path = std::env::join_paths(
        std::iter::once(binary.parent().unwrap().to_path_buf())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let result = Command::new("bash")
        .env("PATH", path)
        .args([
            "--noprofile", "--norc", "-c",
            "eval \"$(muhan completions bash)\" || exit; COMP_WORDS=(muhan experience ap); COMP_CWORD=2; COMP_TYPE=9; _clap_complete_muhan muhan ap experience; printf '%s\\n' \"${COMPREPLY[@]}\"",
        ])
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "apera-ai");
}
