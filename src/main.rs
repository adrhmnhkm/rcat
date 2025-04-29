// src/main.rs

// --- Impor Crate dan Modul Standar ---
use clap::Parser;                      // Untuk parsing argumen command-line
use once_cell::sync::Lazy;             // Untuk inisialisasi lazy static
use std::fs::File;                     // Untuk membuka file
use std::io::{self, BufRead, BufReader}; // Untuk I/O, terutama membaca per baris
use std::path::Path;                   // Untuk bekerja dengan path file (mendapatkan ekstensi)
use std::process;                      // Untuk keluar program (process::exit)
use syntect::easy::HighlightLines;     // Helper syntect untuk highlighting per baris
use syntect::highlighting::{Theme, ThemeSet, Style}; // Komponen syntect untuk tema dan style
use syntect::parsing::SyntaxSet;       // Komponen syntect untuk definisi sintaks
use syntect::util::as_24_bit_terminal_escaped; // Helper syntect untuk output terminal berwarna


// --- Definisi Global (Lazy Static) ---
// Memuat definisi sintaks dan tema hanya sekali saat pertama kali dibutuhkan
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    SyntaxSet::load_defaults_newlines()
});
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    ThemeSet::load_defaults()
});


// --- Definisi Struktur Argumen CLI menggunakan clap ---
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "rcat: Alternatif 'cat' modern ditulis dengan Rust.",
    long_about = None // Bisa ditambahkan deskripsi panjang jika perlu
)]
struct Cli {
    /// Tampilkan nomor pada setiap baris output
    #[arg(short, long)] // -n, --number
    number: bool,

    /// Nama tema syntax highlighting yang ingin digunakan (lihat --list-themes)
    #[arg(long)] // --theme <NAMA>
    theme: Option<String>, // Opsional

    /// Tampilkan daftar semua tema yang tersedia dan keluar
    #[arg(long)] // --list-themes
    list_themes: bool,

    /// Daftar file yang akan diproses (jika kosong, baca dari stdin)
    #[arg()]
    files: Vec<String>,
}


// --- Fungsi Helper untuk Mendapatkan Tema ---
// Mengembalikan referensi ke tema berdasarkan nama, dengan fallback ke default
fn get_theme<'a>(theme_name: &str) -> &'a Theme {
    THEME_SET.themes.get(theme_name)
        .unwrap_or_else(|| {
            eprintln!(
                "rcat: Peringatan: Tema '{}' tidak ditemukan. Menggunakan default 'base16-ocean.dark'.",
                theme_name
            );
            // Fallback ke tema default yang pasti ada
            &THEME_SET.themes["base16-ocean.dark"]
        })
}


// --- Fungsi untuk Memproses Satu File ---
fn process_file(filename: &str, number_lines: bool, theme_name: &str) -> io::Result<()> {
    let path = Path::new(filename);

    // Deteksi sintaks berdasarkan ekstensi file, fallback ke plain text
    let syntax = SYNTAX_SET.find_syntax_by_extension(
        path.extension().and_then(|s| s.to_str()).unwrap_or("")
    ).unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    // Dapatkan tema menggunakan fungsi helper (sudah handle fallback)
    let theme = get_theme(theme_name);

    // Buat instance highlighter
    let mut highlighter = HighlightLines::new(syntax, theme);

    // Buka file dan siapkan reader
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut line_num = 1; // Counter nomor baris

    // Proses baris per baris
    for line_result in reader.lines() {
        let line = line_result?; // Tangani error pembacaan baris

        // Lakukan highlighting per baris
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(&line, &SYNTAX_SET)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Syntect error: {}", e)))?; // Konversi error syntect ke io::Error

        // Cetak nomor baris jika flag -n aktif
        if number_lines {
            print!("{:>6}\t", line_num); // Rata kanan, lebar 6, diikuti tab
        }

        // Cetak segmen baris yang sudah diwarnai
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], true));

        // Tambahkan newline kembali (karena .lines() menghilangkannya)
        println!();

        // Increment nomor baris
        line_num += 1;
    }

    Ok(()) // Kembalikan Ok jika sukses
}


// --- Fungsi untuk Memproses Standard Input ---
fn process_stdin(number_lines: bool, theme_name: &str) -> io::Result<()> {
    // Pesan info bahwa program menunggu input
    eprintln!("(Membaca dari standard input. Tekan Ctrl+D untuk selesai)");

    // Asumsikan stdin adalah plain text
    let syntax = SYNTAX_SET.find_syntax_plain_text();

    // Dapatkan tema menggunakan fungsi helper
    let theme = get_theme(theme_name);

    // Buat instance highlighter untuk plain text
    let mut highlighter = HighlightLines::new(syntax, theme);

    // Dapatkan handle ke stdin dan kunci untuk pembacaan
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut line_num = 1; // Counter nomor baris
    let mut line_buffer = String::new(); // Buffer untuk menampung tiap baris

    // Baca dari stdin baris per baris sampai EOF
    while handle.read_line(&mut line_buffer)? > 0 {
        // Hilangkan newline di akhir sebelum highlighting
        let trimmed_line = line_buffer.trim_end_matches('\n');

        // Highlight baris (sebagai plain text)
        let ranges: Vec<(Style, &str)> = highlighter
            .highlight_line(trimmed_line, &SYNTAX_SET)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Syntect error: {}", e)))?; // Konversi error syntect ke io::Error

        // Cetak nomor baris jika flag -n aktif
        if number_lines {
            print!("{:>6}\t", line_num);
        }

        // Cetak segmen baris yang sudah (atau tidak) di-highlight
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], true));

        // Tambahkan newline kembali
        println!();

        // Naikkan nomor baris
        line_num += 1;
        // Kosongkan buffer untuk baris berikutnya
        line_buffer.clear();
    }

    Ok(()) // Kembalikan Ok jika sukses
}


// --- Fungsi Utama (main) ---
fn main() {
    // Parse argumen command line menggunakan clap
    let cli = Cli::parse();

    // --- Handle jika pengguna meminta daftar tema ---
    if cli.list_themes {
        println!("Tema yang tersedia:");
        // Iterasi melalui nama-nama tema yang dimuat
        for theme_name in THEME_SET.themes.keys() {
            println!("- {}", theme_name);
        }
        // Keluar program setelah mencetak daftar
        process::exit(0);
    }

    // Flag untuk melacak jika terjadi error saat memproses file/stdin
    let mut any_error_occurred = false;

    // Tentukan nama tema yang akan digunakan (dari argumen atau default)
    let theme_name_to_use = cli.theme.as_deref().unwrap_or("base16-ocean.dark");

    // --- Logika utama: proses stdin atau file ---
    if cli.files.is_empty() {
        // Jika tidak ada argumen file, proses standard input
        match process_stdin(cli.number, theme_name_to_use) {
            Ok(_) => {} // Sukses
            Err(e) => {
                eprintln!("rcat: Gagal membaca standard input: {}", e);
                any_error_occurred = true; // Tandai ada error
            }
        }
    } else {
        // Jika ada argumen file, proses setiap file dalam loop
        for filename in &cli.files {
            match process_file(filename, cli.number, theme_name_to_use) {
                Ok(_) => {} // Sukses untuk file ini
                Err(e) => {
                    // Cetak error spesifik untuk file yang gagal
                    eprintln!("rcat: {}: {}", filename, e);
                    any_error_occurred = true; // Tandai ada error
                }
            }
        }
    }

    // --- Tentukan status keluar program ---
    // Jika ada error yang terjadi selama eksekusi, keluar dengan status 1
    if any_error_occurred {
        process::exit(1);
    }
    // Jika tidak ada error, program akan keluar secara normal dengan status 0
}