use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StyleMode {
    /// Teks saja (tanpa header, nomor baris, atau pembatas garis)
    Plain,
    /// Hanya nomor baris seperti cat klasik
    Numbers,
    /// Nomor baris dengan pembatas garis vertikal (gutter)
    Grid,
    /// Header nama file dengan teks tanpa nomor baris
    Header,
    /// Tampilan lengkap: header nama file, nomor baris, dan grid
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if let Some((start_str, end_str)) = s.split_once(':') {
            let start = if start_str.trim().is_empty() {
                1
            } else {
                let n = start_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| format!("Nomor baris awal tidak valid '{}': {}", start_str, e))?;
                if n == 0 { 1 } else { n }
            };

            let end = if end_str.trim().is_empty() {
                usize::MAX
            } else {
                end_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|e| format!("Nomor baris akhir tidak valid '{}': {}", end_str, e))?
            };

            if start > end {
                return Err(format!(
                    "Baris awal ({}) tidak boleh lebih besar dari baris akhir ({})",
                    start, end
                ));
            }

            Ok(Self { start, end })
        } else {
            let line = s
                .parse::<usize>()
                .map_err(|e| format!("Nomor baris tidak valid '{}': {}", s, e))?;
            let line = if line == 0 { 1 } else { line };
            Ok(Self {
                start: line,
                end: line,
            })
        }
    }

    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "rcat",
    author,
    version,
    about = "rcat: Alternatif modern 'cat' ditulis dengan Rust dengan syntax highlighting dan berbagai fitur canggih.",
    long_about = None
)]
pub struct Cli {
    /// Tampilkan nomor pada setiap baris output
    #[arg(short = 'n', long = "number")]
    pub number: bool,

    /// Tampilkan nomor baris hanya untuk baris yang tidak kosong (mengabaikan -n)
    #[arg(short = 'b', long = "number-nonblank")]
    pub number_nonblank: bool,

    /// Gabungkan beberapa baris kosong yang berurutan menjadi 1 baris kosong saja
    #[arg(short = 's', long = "squeeze-blank")]
    pub squeeze_blank: bool,

    /// Tampilkan karakter '$' di akhir setiap baris
    #[arg(short = 'E', long = "show-ends")]
    pub show_ends: bool,

    /// Tampilkan karakter TAB sebagai '^I'
    #[arg(short = 'T', long = "show-tabs")]
    pub show_tabs: bool,

    /// Ekuivalen dengan -ET (tampilkan ends dan tabs)
    #[arg(short = 'A', long = "show-all")]
    pub show_all: bool,

    /// Paksa bahasa syntax highlighting tertentu (contoh: rust, py, js, md)
    #[arg(short = 'l', long = "language")]
    pub language: Option<String>,

    /// Cetak hanya rentang baris tertentu (contoh: '10:25', '10:', ':50', atau '42')
    #[arg(short = 'r', long = "line-range", value_parser = LineRange::parse)]
    pub line_range: Option<LineRange>,

    /// Mode tampilan styling
    #[arg(long = "style", value_enum)]
    pub style: Option<StyleMode>,

    /// Nama tema syntax highlighting yang ingin digunakan (lihat --list-themes)
    #[arg(long = "theme")]
    pub theme: Option<String>,

    /// Tampilkan daftar semua tema yang tersedia dan keluar
    #[arg(long = "list-themes")]
    pub list_themes: bool,

    /// Tampilkan daftar semua bahasa sintaks yang didukung dan keluar
    #[arg(long = "list-languages")]
    pub list_languages: bool,

    /// Tampilkan file dalam format hex dump
    #[arg(short = 'X', long = "hex")]
    pub hex: bool,

    /// Paksa tampilkan sebagai teks meskipun file terdeteksi biner
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Path kustom ke file konfigurasi toml
    #[arg(long = "config")]
    pub config: Option<std::path::PathBuf>,

    /// Abaikan file konfigurasi bawaan
    #[arg(long = "no-config")]
    pub no_config: bool,

    /// Aktifkan indikator perubahan git diff di samping nomor baris
    #[arg(long = "git")]
    pub git: bool,

    /// Nonaktifkan indikator git diff
    #[arg(long = "no-git")]
    pub no_git: bool,

    /// Daftar file yang akan diproses (jika kosong, baca dari stdin)
    #[arg()]
    pub files: Vec<String>,
}

impl Cli {
    /// Menentukan gaya tampilan efektif berdasarkan argumen
    pub fn effective_style(&self) -> StyleMode {
        if let Some(style) = self.style {
            style
        } else if self.number || self.number_nonblank {
            StyleMode::Grid
        } else {
            StyleMode::Plain
        }
    }

    /// Terapkan konfigurasi file ke argumen CLI (CLI memiliki prioritas lebih tinggi)
    pub fn apply_config(&mut self, config: &crate::config::Config) {
        if self.theme.is_none() && config.theme.is_some() {
            self.theme = config.theme.clone();
        }

        if self.style.is_none() && let Some(ref style_str) = config.style {
            match style_str.to_lowercase().as_str() {
                "plain" => self.style = Some(StyleMode::Plain),
                "numbers" => self.style = Some(StyleMode::Numbers),
                "grid" => self.style = Some(StyleMode::Grid),
                "header" => self.style = Some(StyleMode::Header),
                "full" => self.style = Some(StyleMode::Full),
                _ => {}
            }
        }

        if !self.number && config.number == Some(true) {
            self.number = true;
        }

        if !self.squeeze_blank && config.squeeze_blank == Some(true) {
            self.squeeze_blank = true;
        }

        if !self.show_ends && config.show_ends == Some(true) {
            self.show_ends = true;
        }

        if !self.show_tabs && config.show_tabs == Some(true) {
            self.show_tabs = true;
        }
    }

    /// Apakah nomor baris harus ditampilkan
    pub fn should_number(&self) -> bool {
        self.number
            || self.number_nonblank
            || matches!(
                self.effective_style(),
                StyleMode::Numbers | StyleMode::Grid | StyleMode::Full
            )
    }

    /// Apakah karakter akhir baris ($) harus ditampilkan
    pub fn display_ends(&self) -> bool {
        self.show_ends || self.show_all
    }

    /// Apakah karakter tab (^I) harus ditampilkan
    pub fn display_tabs(&self) -> bool {
        self.show_tabs || self.show_all
    }

    /// Apakah indikator git diff harus diaktifkan
    pub fn is_git_enabled(&self) -> bool {
        if self.no_git {
            false
        } else if self.git {
            true
        } else {
            matches!(self.effective_style(), StyleMode::Grid | StyleMode::Full)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_range_parsing() {
        let r1 = LineRange::parse("10:20").unwrap();
        assert_eq!(r1.start, 10);
        assert_eq!(r1.end, 20);
        assert!(r1.contains(10));
        assert!(r1.contains(15));
        assert!(r1.contains(20));
        assert!(!r1.contains(9));
        assert!(!r1.contains(21));

        let r2 = LineRange::parse(":50").unwrap();
        assert_eq!(r2.start, 1);
        assert_eq!(r2.end, 50);

        let r3 = LineRange::parse("15:").unwrap();
        assert_eq!(r3.start, 15);
        assert_eq!(r3.end, usize::MAX);

        let r4 = LineRange::parse("42").unwrap();
        assert_eq!(r4.start, 42);
        assert_eq!(r4.end, 42);

        assert!(LineRange::parse("50:20").is_err());
        assert!(LineRange::parse("invalid").is_err());
    }
}

