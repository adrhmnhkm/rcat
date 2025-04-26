// src/main.rs

use clap::Parser; // Import clap Parser trait
use std::fs::File;
// Tambahkan BufRead dan BufReader untuk membaca baris per baris secara efisien
use std::io::{self, BufRead, BufReader};
use std::process;

// --- Definisi Argumen CLI menggunakan clap ---
#[derive(Parser, Debug)]
#[command(author, version, about = "Alternatif 'cat' ditulis dengan Rust", long_about = None)] // Menambahkan info help
struct Cli {
    /// Opsi untuk menampilkan nomor pada setiap baris output
    #[arg(short, long)] // Mendefinisikan flag -n dan --number
    number: bool,

    /// Daftar file yang akan diproses
    #[arg()] // Argumen posisi untuk nama file
    files: Vec<String>,
}
// --- Akhir Definisi Argumen CLI ---


// --- Modifikasi Fungsi process_file ---
// Sekarang menerima flag 'number_lines' dan membaca baris per baris
fn process_file(filename: &str, number_lines: bool) -> io::Result<()> {
    let file = File::open(filename)?;
    // Gunakan BufReader untuk efisiensi saat membaca baris per baris
    let reader = BufReader::new(file);

    let mut line_num = 1; // Inisialisasi nomor baris

    // Iterasi melalui setiap baris dalam file
    // reader.lines() mengembalikan iterator atas Result<String, io::Error>
    for line_result in reader.lines() {
        // Ambil String dari Result, gunakan '?' untuk propagate error jika ada
        let line = line_result?;

        // Cek apakah flag penomoran aktif
        if number_lines {
            // Cetak nomor baris (rata kanan, lebar 6 char), tab, lalu isi baris
            // Kita pakai println! karena reader.lines() menghilangkan newline, jadi kita tambahkan lagi.
            println!("{:>6}\t{}", line_num, line);
        } else {
            // Jika tidak ada penomoran, cukup cetak barisnya
            println!("{}", line);
        }
        // Naikkan nomor baris untuk iterasi berikutnya
        line_num += 1;
    }

    // Jika semua baris berhasil diproses, kembalikan Ok
    Ok(())
}
// --- Akhir Modifikasi Fungsi process_file ---


fn main() {
    // Parse argumen baris perintah menggunakan definisi struct Cli
    let cli = Cli::parse();

    // Cek apakah ada file yang diberikan sebagai argumen
    if cli.files.is_empty() {
        // Di masa depan, kita bisa tambahkan logika untuk membaca dari stdin di sini
        eprintln!("rcat: Tidak ada file input yang diberikan.");
        // Clap secara otomatis bisa generate pesan penggunaan, tapi ini contoh manual
        eprintln!("Untuk bantuan, coba: rcat --help");
        process::exit(1);
    }

    let mut any_error_occurred = false;

    // Loop melalui daftar file yang didapat dari clap
    for filename in &cli.files {
        // Panggil process_file, teruskan nilai flag 'number' dari cli
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