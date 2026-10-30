use std::io::{self, Read, Write};

const COLOR_GRAY: &str = "\x1b[90m";
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_YELLOW: &str = "\x1b[33m";

/// Memeriksa apakah buffer berisi data biner
pub fn is_binary_buffer(buf: &[u8]) -> bool {
    if buf.is_empty() {
        return false;
    }
    // Cek null byte dalam 1024 byte pertama
    let sample = &buf[..buf.len().min(1024)];
    if sample.contains(&0) {
        return true;
    }
    // Cek persentase karakter kontrol non-text
    let non_text = sample
        .iter()
        .filter(|&&b| b < 9 || (b > 13 && b < 32) || b == 127)
        .count();

    (non_text as f32 / sample.len() as f32) > 0.10
}

/// Cetak representasi Hex Dump dari reader
pub fn print_hex_dump<R: Read, W: Write>(
    mut reader: R,
    writer: &mut W,
    file_label: Option<&str>,
    full_style: bool,
) -> io::Result<()> {
    if full_style {
        let label = file_label.unwrap_or("stdin");
        writeln!(
            writer,
            "{}────────┬────────────────────────────────────────────────────────────────────────{}",
            COLOR_GRAY, COLOR_RESET
        )?;
        writeln!(
            writer,
            "{}        │{} {}{} (Hex Dump){}",
            COLOR_GRAY, COLOR_RESET, COLOR_CYAN, label, COLOR_RESET
        )?;
        writeln!(
            writer,
            "{}────────┼────────────────────────────────────────────────────────────────────────{}",
            COLOR_GRAY, COLOR_RESET
        )?;
    }

    let mut buffer = [0u8; 16];
    let mut offset: usize = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];

        // 1. Kolom Offset (8 digit hex)
        write!(writer, "{}{:08x}{}  ", COLOR_CYAN, offset, COLOR_RESET)?;

        // 2. Kolom Hex Bytes (16 bytes, dipisah 2 grup @ 8 byte)
        for i in 0..16 {
            if i == 8 {
                write!(writer, " ")?;
            }

            if let Some(&b) = chunk.get(i) {
                if b == 0 {
                    write!(writer, "{}00{} ", COLOR_GRAY, COLOR_RESET)?;
                } else if b.is_ascii_graphic() || b == b' ' {
                    write!(writer, "{}{:02x}{} ", COLOR_GREEN, b, COLOR_RESET)?;
                } else {
                    write!(writer, "{}{:02x}{} ", COLOR_YELLOW, b, COLOR_RESET)?;
                }
            } else {
                // Padding spasi jika di akhir file kurang dari 16 byte
                write!(writer, "   ")?;
            }
        }

        // 3. Kolom Karakter ASCII
        write!(writer, " {}│{}", COLOR_GRAY, COLOR_RESET)?;
        for &b in chunk {
            if b.is_ascii_graphic() || b == b' ' {
                write!(writer, "{}{}{}", COLOR_GREEN, b as char, COLOR_RESET)?;
            } else if b == 0 {
                write!(writer, "{}.{}", COLOR_GRAY, COLOR_RESET)?;
            } else {
                write!(writer, "{}.{}", COLOR_YELLOW, COLOR_RESET)?;
            }
        }
        writeln!(writer, "{}│{}", COLOR_GRAY, COLOR_RESET)?;

        offset += bytes_read;
    }

    if full_style {
        writeln!(
            writer,
            "{}────────┴────────────────────────────────────────────────────────────────────────{}",
            COLOR_GRAY, COLOR_RESET
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_binary_detection() {
        let text_data = b"Hello, world! This is plain text.\nWith multiple lines.";
        assert!(!is_binary_buffer(text_data));

        let binary_null = b"Hello\x00World";
        assert!(is_binary_buffer(binary_null));

        let elf_header = b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        assert!(is_binary_buffer(elf_header));
    }

    #[test]
    fn test_hex_dump_output() {
        let data = b"Test 1234\x00\x01\x02\x03\x04\x05";
        let mut output = Vec::new();
        print_hex_dump(&data[..], &mut output, Some("test.bin"), false).unwrap();
        let res = String::from_utf8_lossy(&output);
        assert!(res.contains("00000000"));
        // 54 65 73 74 adalah representasi hex dari 'T' 'e' 's' 't'
        assert!(res.contains("54"));
        assert!(res.contains("65"));
        assert!(res.contains("73"));
        assert!(res.contains("74"));
    }
}
