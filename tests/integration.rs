use std::process::Command;

fn mpush_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mpush"))
}

#[test]
fn test_help_flag() {
    let output = mpush_bin().arg("-h").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: mpush"));
    assert!(stdout.contains("MPUSH_APP_ID"));
}

#[test]
fn test_missing_required_args() {
    let output = mpush_bin().output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn test_missing_env_vars() {
    let output = mpush_bin()
        .env_remove("MPUSH_APP_ID")
        .env_remove("MPUSH_APP_SECRET")
        .args(["-u", "test", "-t", "test", "-d", "test"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
#[ignore] // requires real credentials: MPUSH_APP_ID, MPUSH_APP_SECRET, MPUSH_TEST_OPENID, MPUSH_TEST_TEMPLATE_ID
fn test_send_real_message() {
    let openid = std::env::var("MPUSH_TEST_OPENID").unwrap();
    let template_id = std::env::var("MPUSH_TEST_TEMPLATE_ID").unwrap();
    let output = mpush_bin()
        .args(["-u", &openid, "-t", &template_id, "-d", "integration test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    // stdout should be empty (no output on success)
    assert!(output.stdout.is_empty());
}
