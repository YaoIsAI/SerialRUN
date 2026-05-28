use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_list_command() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_list_json_format() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .args(["list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ports"));
}

#[test]
fn test_help_command() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SerialRUN"));
}

#[test]
fn test_version_command() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("serialrun"));
}

#[test]
fn test_connect_help() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .args(["connect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("baud"));
}

#[test]
fn test_send_help() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .args(["send", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hex"));
}

#[test]
fn test_monitor_help() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .args(["monitor", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("timestamp"));
}

#[test]
fn test_agent_help() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .args(["agent", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list-ports"));
}

#[test]
fn test_agent_list_ports() {
    Command::cargo_bin("serialrun")
        .unwrap()
        .args(["agent", "list-ports"])
        .assert()
        .success()
        .stdout(predicate::str::contains("success"));
}
