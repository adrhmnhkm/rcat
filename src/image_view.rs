use image::GenericImageView;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::Path;

const COLOR_GRAY: &str = "\x1b[90m";
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_CYAN: &str = "\x1b[36m";

/// Memeriksa apakah file adalah gambar berdasarkan ekstensi atau magic bytes
pub fn is_image_file(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico"
    ) {
        return true;
    }

    // Coba baca magic bytes jika ada file
    if let Ok(mut f) = File::open(path) {
        let mut magic = [0u8; 8];
        if f.read(&mut magic).unwrap_or(0) >= 4 {
            // PNG magic bytes
            if &magic[0..4] == b"\x89PNG" {
                return true;
            }
            // JPEG magic bytes
            if &magic[0..3] == b"\xff\xd8\xff" {
                return true;
            }
            // GIF magic bytes
            if &magic[0..4] == b"GIF8" {
                return true;
            }
            // BMP magic bytes
            if &magic[0..2] == b"BM" {
                return true;
            }
        }
    }

    false
}

/// Merender gambar langsung ke terminal menggunakan karakter Unicode half-block (▀)
pub fn print_image_preview<W: Write>(
    path: &Path,
    writer: &mut W,
    max_width: u32,
    full_style: bool,
) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let img = match image::load(reader, image::ImageFormat::from_path(path).unwrap_or(image::ImageFormat::Png)) {
        Ok(i) => i,
        Err(e) => {
            return Err(io::Error::other(format!("Gagal memuat gambar: {}", e)));
        }
    };

    let (orig_w, orig_h) = img.dimensions();

    // Hitung ukuran tampilan terminal (1 kolom karakter = 2 baris pixel)
    let target_w = if orig_w > max_width {
        max_width
    } else {
        orig_w.max(20)
    };
    let aspect_ratio = orig_h as f32 / orig_w as f32;
    // Kalikan 2 karena 1 baris terminal muat 2 baris pixel (top & bottom half block)
    let target_h = ((target_w as f32 * aspect_ratio) * 2.0).round() as u32;
    // Pastikan genap agar setiap block memiliki top dan bottom
    let target_h = if !target_h.is_multiple_of(2) { target_h + 1 } else { target_h };

    let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle);
    let rgba = resized.to_rgba8();

    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("image");

    if full_style {
        writeln!(
            writer,
            "{}───────┬──────────────────────────────────────────────────────────────────{}",
            COLOR_GRAY, COLOR_RESET
        )?;
        writeln!(
            writer,
            "{}       │{} {}{} ({}x{} px){}",
            COLOR_GRAY, COLOR_RESET, COLOR_CYAN, filename, orig_w, orig_h, COLOR_RESET
        )?;
        writeln!(
            writer,
            "{}───────┼──────────────────────────────────────────────────────────────────{}",
            COLOR_GRAY, COLOR_RESET
        )?;
    }

    // Render baris per baris terminal (2 baris pixel per baris teks)
    for y in (0..target_h).step_by(2) {
        if full_style {
            write!(writer, "{}       │{} ", COLOR_GRAY, COLOR_RESET)?;
        }

        for x in 0..target_w {
            let top_pixel = rgba.get_pixel(x, y);
            let bottom_pixel = if y + 1 < target_h {
                rgba.get_pixel(x, y + 1)
            } else {
                top_pixel
            };

            // Jika top pixel transparan
            let top_alpha = top_pixel[3];
            let bottom_alpha = bottom_pixel[3];

            if top_alpha < 32 && bottom_alpha < 32 {
                write!(writer, " ")?;
            } else if bottom_alpha < 32 {
                // Hanya top pixel
                write!(
                    writer,
                    "\x1b[38;2;{};{};{}m▀{}",
                    top_pixel[0], top_pixel[1], top_pixel[2], COLOR_RESET
                )?;
            } else if top_alpha < 32 {
                // Hanya bottom pixel (gunakan lower half block ▄)
                write!(
                    writer,
                    "\x1b[38;2;{};{};{}m▄{}",
                    bottom_pixel[0], bottom_pixel[1], bottom_pixel[2], COLOR_RESET
                )?;
            } else {
                // Keduanya ada warna: fg = top pixel, bg = bottom pixel, char = ▀
                write!(
                    writer,
                    "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m▀{}",
                    top_pixel[0], top_pixel[1], top_pixel[2],
                    bottom_pixel[0], bottom_pixel[1], bottom_pixel[2],
                    COLOR_RESET
                )?;
            }
        }
        writeln!(writer)?;
    }

    if full_style {
        writeln!(
            writer,
            "{}───────┴──────────────────────────────────────────────────────────────────{}",
            COLOR_GRAY, COLOR_RESET
        )?;
    }

    Ok(())
}

