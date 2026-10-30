use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn rcat_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rcat"))
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b'
            && chars.peek() == Some(&'[')
        {
            chars.next();
            while let Some(&next) = chars.peek() {
                chars.next();
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

#[test]
fn test_list_themes() {
    let output = rcat_cmd().arg("--list-themes").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("base16-ocean.dark"));
    assert!(stdout.contains("Daftar tema yang tersedia:"));
}

#[test]
fn test_list_languages() {
    let output = rcat_cmd().arg("--list-languages").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rust"));
    assert!(stdout.contains("Python"));
}

#[test]
fn test_number_flag() {
    let test_file = "target/test_num.txt";
    fs::write(test_file, "baris 1\nbaris 2\nbaris 3\n").unwrap();

    let output = rcat_cmd().args(["-n", test_file]).output().unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("1 │ baris 1"));
    assert!(stdout.contains("2 │ baris 2"));
    assert!(stdout.contains("3 │ baris 3"));
}

#[test]
fn test_number_nonblank_flag() {
    let test_file = "target/test_nonblank.txt";
    fs::write(test_file, "pertama\n\nkedua\n").unwrap();

    let output = rcat_cmd().args(["-b", test_file]).output().unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("1 │ pertama"));
    assert!(stdout.contains("  │")); // baris kosong tanpa angka
    assert!(stdout.contains("2 │ kedua"));
}

#[test]
fn test_squeeze_blank_flag() {
    let test_file = "target/test_squeeze.txt";
    fs::write(test_file, "awal\n\n\n\nakhir\n").unwrap();

    let output = rcat_cmd().args(["-s", "--style", "plain", test_file]).output().unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["awal", "", "akhir"]);
}

#[test]
fn test_show_ends_and_tabs() {
    let test_file = "target/test_ends_tabs.txt";
    fs::write(test_file, "hello\tworld\n").unwrap();

    let output = rcat_cmd().args(["-A", "--style", "plain", test_file]).output().unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("^I"));
    assert!(stdout.contains('$'));
}

#[test]
fn test_line_range() {
    let test_file = "target/test_range.txt";
    fs::write(test_file, "1\n2\n3\n4\n5\n").unwrap();

    let output = rcat_cmd()
        .args(["-r", "2:4", "--style", "plain", test_file])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["2", "3", "4"]);
}

#[test]
fn test_style_full_header() {
    let test_file = "target/test_full.txt";
    fs::write(test_file, "konten header\n").unwrap();

    let output = rcat_cmd().args(["--style", "full", test_file]).output().unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("target/test_full.txt"));
    assert!(stdout.contains("───────┬──"));
    assert!(stdout.contains("───────┴──"));
}

#[test]
fn test_stdin_input() {
    let mut child = rcat_cmd()
        .args(["--style", "plain"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"halo dari stdin\n").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("halo dari stdin"));
}

#[test]
fn test_binary_file_detection() {
    let test_file = "target/test_bin.bin";
    fs::write(test_file, b"ELF\x00\x01\x02\x03\x04\x05binary_data").unwrap();

    let output = rcat_cmd().arg(test_file).output().unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("terdeteksi sebagai file biner"));
}

#[test]
fn test_hex_dump_flag() {
    let test_file = "target/test_hex.bin";
    fs::write(test_file, b"ABCDEFGH12345678").unwrap();

    let output = rcat_cmd().args(["-X", test_file]).output().unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("00000000"));
    assert!(stdout.contains("41 42 43 44"));
    assert!(stdout.contains("ABCDEFGH12345678"));
}

#[test]
fn test_custom_config_file() {
    let config_file = "target/custom_config.toml";
    fs::write(
        config_file,
        r#"
        style = "full"
        number = true
    "#,
    )
    .unwrap();

    let test_file = "target/test_cfg.txt";
    fs::write(test_file, "baris dari config test\n").unwrap();

    let output = rcat_cmd()
        .args(["--config", config_file, test_file])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = strip_ansi(&String::from_utf8_lossy(&output.stdout));
    // Dari config, style="full" dan number=true
    assert!(stdout.contains("target/test_cfg.txt"));
    assert!(stdout.contains("1 │"));
    assert!(stdout.contains("baris dari config test"));
}
