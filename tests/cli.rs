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
