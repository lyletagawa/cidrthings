use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cidrthings"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).trim().to_owned(),
        String::from_utf8_lossy(&out.stderr).trim().to_owned(),
    )
}

fn run_with_stdin(args: &[&str], input: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cidrthings"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).trim().to_owned(),
        String::from_utf8_lossy(&out.stderr).trim().to_owned(),
    )
}

#[test]
fn two_cidrs() {
    let (code, stdout, _) = run(&["10.1.0.0/24", "10.2.0.0/24"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "10.0.0.0/14");
}

#[test]
fn single_cidr_is_itself() {
    let (code, stdout, _) = run(&["192.168.1.0/24"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "192.168.1.0/24");
}

#[test]
fn three_cidrs() {
    let (code, stdout, _) = run(&["192.168.0.0/24", "192.168.1.0/24", "192.168.2.0/24"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "192.168.0.0/22");
}

#[test]
fn contained_block() {
    let (code, stdout, _) = run(&["10.0.0.0/8", "10.1.0.0/24"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "10.0.0.0/8");
}

#[test]
fn bare_ipv4_host() {
    let (code, stdout, _) = run(&["10.0.0.1", "10.0.0.2"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "10.0.0.0/30");
}

#[test]
fn ipv6() {
    let (code, stdout, _) = run(&["2001:db8::/32", "2001:db9::/32"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "2001:db8::/31");
}

#[test]
fn mixed_families_exits_nonzero() {
    let (code, _, stderr) = run(&["10.0.0.0/8", "2001:db8::/32"]);
    assert_ne!(code, 0);
    assert!(!stderr.is_empty());
}

#[test]
fn invalid_cidr_exits_nonzero() {
    let (code, _, stderr) = run(&["not-a-cidr"]);
    assert_ne!(code, 0);
    assert!(!stderr.is_empty());
}

#[test]
fn version_flag() {
    let (code, stdout, _) = run(&["--version"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("0.2.0"));
}

#[test]
fn summarize_flag_two_groups() {
    let (code, stdout, _) = run(&[
        "--summarize",
        "10.0.0.0/24",
        "10.0.1.0/24",
        "192.168.0.0/24",
        "192.168.1.0/24",
    ]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "10.0.0.0/23");
    assert_eq!(lines[1], "192.168.0.0/23");
}

#[test]
fn summarize_flag_single_group() {
    let (code, stdout, _) = run(&["--summarize", "10.0.0.0/24", "10.0.1.0/24"]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "10.0.0.0/23");
}

#[test]
fn summarize_short_flag() {
    let (code, stdout, _) = run(&["-s", "10.0.0.0/24", "10.0.1.0/24", "192.168.0.0/24"]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "10.0.0.0/23");
    assert_eq!(lines[1], "192.168.0.0/24");
}

#[test]
fn stdin_basic() {
    let (code, stdout, _) = run_with_stdin(&[], "10.1.0.0/24\n10.2.0.0/24\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "10.0.0.0/14");
}

#[test]
fn stdin_comma_separated() {
    let (code, stdout, _) = run_with_stdin(&[], "10.1.0.0/24,10.2.0.0/24\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "10.0.0.0/14");
}

#[test]
fn stdin_with_summarize_flag() {
    let (code, stdout, _) = run_with_stdin(
        &["--summarize"],
        "10.0.0.0/24\n10.0.1.0/24\n192.168.0.0/24\n",
    );
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "10.0.0.0/23");
    assert_eq!(lines[1], "192.168.0.0/24");
}

#[test]
fn stdin_merged_with_args() {
    let (code, stdout, _) = run_with_stdin(&["10.1.0.0/24"], "10.2.0.0/24\n");
    assert_eq!(code, 0);
    assert_eq!(stdout, "10.0.0.0/14");
}
