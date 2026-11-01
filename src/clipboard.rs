use base64::Engine;
use std::io::{self, Write};

/// Menyalin teks ke clipboard menggunakan OSC 52 sequence dan fallback native
pub fn copy_to_clipboard(text: &str) -> io::Result<()> {
    // 1. Kirim OSC 52 escape code ke terminal (kompatibel lintas platform: Windows Terminal, Alacritty, Kitty, WezTerm, SSH)
    let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    let osc52 = format!("\x1b]52;c;{}\x07", b64);
    let mut stderr = io::stderr().lock();
    let _ = stderr.write_all(osc52.as_bytes());
    let _ = stderr.flush();

    // 2. Fallback native di Windows
    #[cfg(windows)]
    {
        use std::process::{Command, Stdio};
        if let Ok(mut child) = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Set-Clipboard -Value $input"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }

    Ok(())
}

