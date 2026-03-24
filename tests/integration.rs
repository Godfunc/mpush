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

#[test]
fn test_empty_openid_segment() {
    let output = mpush_bin()
        .args(["-u", "id1,,id2", "-t", "tmpl1", "-d", "hello"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty openid"));
}

#[test]
fn test_trailing_comma_openid() {
    let output = mpush_bin()
        .args(["-u", "id1,", "-t", "tmpl1", "-d", "hello"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty openid"));
}

#[test]
fn test_leading_comma_openid() {
    let output = mpush_bin()
        .args(["-u", ",id1", "-t", "tmpl1", "-d", "hello"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("empty openid"));
}

#[test]
#[ignore] // requires real credentials: MPUSH_APP_ID, MPUSH_APP_SECRET, MPUSH_TEST_OPENID, MPUSH_TEST_TEMPLATE_ID
fn test_send_multiple_users_all_succeed() {
    let openid = std::env::var("MPUSH_TEST_OPENID").unwrap();
    let template_id = std::env::var("MPUSH_TEST_TEMPLATE_ID").unwrap();
    // Send to same user twice (duplicates allowed per spec)
    let users = format!("{openid},{openid}");
    let output = mpush_bin()
        .args(["-u", &users, "-t", &template_id, "-d", "multi-user test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Multi-user mode: should have two [OK] lines
    assert_eq!(stderr.matches("[OK]").count(), 2);
}

#[test]
#[ignore] // requires real credentials
fn test_send_multiple_users_partial_failure() {
    let openid = std::env::var("MPUSH_TEST_OPENID").unwrap();
    let template_id = std::env::var("MPUSH_TEST_TEMPLATE_ID").unwrap();
    let users = format!("{openid},INVALID_OPENID");
    let output = mpush_bin()
        .args(["-u", &users, "-t", &template_id, "-d", "partial fail test"])
        .output()
        .unwrap();
    // Should exit with error code because one user failed
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[OK]"));
    assert!(stderr.contains("[ERR]"));
}

#[test]
#[ignore] // requires real credentials
fn test_send_single_user_backward_compatible() {
    let openid = std::env::var("MPUSH_TEST_OPENID").unwrap();
    let template_id = std::env::var("MPUSH_TEST_TEMPLATE_ID").unwrap();
    let output = mpush_bin()
        .args(["-u", &openid, "-t", &template_id, "-d", "single user test"])
        .output()
        .unwrap();
    assert!(output.status.success());
    // Single user: stdout empty, no [OK]/[ERR] on stderr
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("[OK]"));
    assert!(!stderr.contains("[ERR]"));
}
