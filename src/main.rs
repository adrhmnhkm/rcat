// src/main.rs

use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;
use std::path::Path; 

// Tambah use statement buat syntect dan once_cell 
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::as_24_bit_terminal_escaped; // konversi ke ANSI escape code
use once_cell::sync::Lazy; // inisialisasi lazy static


// --- Definisi Global Lazy Static untuk SyntaxSet dan ThemeSet ---
// Mastiin hanya memuat definisi sintaks dan tema sekali saja
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| {
    // Memuat definisi sintaks bawaan syntect (termasuk banyak bahasa populer)
    SyntaxSet::load_defaults_newlines()
});
static THEME_SET: Lazy<ThemeSet> = Lazy::new(|| {
    // Load tema warna bawaan syntect
    ThemeSet::load_defaults()
});



#[derive(Parser, Debug)]
#[command(author, version, about = "Alternatif 'cat' ditulis dengan Rust", long_about = None)]
struct Cli {
    #[arg(short, long)]
    number: bool,

    #[arg()]
    files: Vec<String>,

    // TODO: Tambahkan opsi untuk memilih tema nanti?
    // #[arg(long, default_value = "base16-ocean.dark")]
    // theme: String,
}


// --- Modifikasi Besar pada Fungsi process_file ---
fn process_file(filename: &str, number_lines: bool) -> io::Result<()> {
    // --- Deteksi Sintaks ---
    let path = Path::new(filename);
    // Coba tebak sintaks berdasarkan ekstensi file
    let syntax = SYNTAX_SET.find_syntax_by_extension(
        path.extension()            // Dapatkan ekstensi file (Option<&OsStr>)
            .and_then(|s| s.to_str()) // Konversi ke Option<&str>
            .unwrap_or("")           // Jika tidak ada ekstensi, pakai string kosong
    )
    // Jika tidak terdeteksi berdasarkan ekstensi, coba tebak dari baris pertama (opsional, bisa dilewati)
    // .or_else(|| SYNTAX_SET.find_syntax_by_first_line(isi_baris_pertama))
    // Jika tetap tidak terdeteksi, gunakan sintaks plain text
    .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    // --- Pilih Tema ---
    // Untuk saat ini, kita hardcode tema populer. Nanti bisa dibuat opsi.
    // Pastikan tema ini ada di ThemeSet::load_defaults()
    let theme_name = "base16-ocean.dark";
    let theme = THEME_SET.themes.get(theme_name).unwrap_or_else(|| {
        eprintln!("Peringatan: Tema '{}' tidak ditemukan, menggunakan tema default pertama.", theme_name);
        &THEME_SET.themes.values().next().unwrap() // Fallback ke tema pertama jika pilihan kita tidak ada
    });


    // --- Persiapan Highlighting ---
    // Membuat instance highlighter untuk file ini
    let mut highlighter = HighlightLines::new(syntax, theme);

    // --- Membaca dan Mencetak File Baris per Baris dengan Highlighting ---
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut line_num = 1;

    for line_result in reader.lines() {
        let line = line_result?;

        // Highlight satu baris. Hasilnya adalah Vec dari tuple (Style, &str segment)
        // Kita gunakan unwrap() di sini untuk simplifikasi, idealnya tangani error parsing
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(&line, &SYNTAX_SET).unwrap();

        // 1. Cetak nomor baris jika diminta (sebelum mencetak isi baris)
        if number_lines {
            print!("{:>6}\t", line_num);
        }

        // 2. Cetak segmen-segmen baris yang sudah diberi style (warna)
        //    Fungsi as_24_bit_terminal_escaped mengonversi Vec<(Style, &str)>
        //    menjadi string dengan ANSI escape codes untuk warna 24-bit (true color).
        //    Parameter kedua (true) berarti kita juga ingin warna background diterapkan.
        print!("{}", as_24_bit_terminal_escaped(&ranges[..], true));

        // 3. Tambahkan newline secara manual
        //    Karena reader.lines() menghilangkan newline dan as_24_bit_terminal_escaped
        //    tidak menambahkannya kembali, kita perlu menambahkannya agar format tetap benar.
        println!(); // Cetak newline kosong

        // Naikkan nomor baris
        line_num += 1;
    }

    Ok(())
}
// --- Akhir Modifikasi Besar process_file ---


fn main() {
    let cli = Cli::parse();

    if cli.files.is_empty() {
        eprintln!("rcat: Tidak ada file input yang diberikan.");
        eprintln!("Untuk bantuan, coba: rcat --help");
        process::exit(1);
    }

    let mut any_error_occurred = false;

    for filename in &cli.files {
        // Panggil process_file seperti biasa, flag number diteruskan
        match process_file(filename, cli.number) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("rcat: {}: {}", filename, e);
                any_error_occurred = true;
            }
        }
    }

    if any_error_occurred {
        process::exit(1);
    }
}