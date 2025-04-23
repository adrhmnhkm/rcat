// src/main.rs

use std::env;
use std::fs::File;
use std::io::{self, Read}; // io diperlukan untuk io::Result
use std::process;

// --- Fungsi Baru ---
// Fungsi ini mengambil nama file sebagai input,
// mencoba membaca dan mencetak isinya.
// Mengembalikan io::Result<()>: Ok(()) jika sukses, Err(e) jika gagal.
fn process_file(filename: &str) -> io::Result<()> {
    // Logika ini sama seperti yang ada di main sebelumnya
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    print!("{}", contents);
    Ok(())
}
// --- Akhir Fungsi Baru ---

// Fungsi main tidak lagi mengembalikan io::Result<()>,
// karena kita ingin kontrol lebih detail atas status keluarnya.
fn main() {
    let args: Vec<String> = env::args().collect();

    // Sedikit ubah pesan penggunaan untuk mencerminkan bisa banyak file
    if args.len() < 2 {
        eprintln!("Penggunaan: rcat <nama_file1> [nama_file2] ...");
        process::exit(1);
    }

    // Flag untuk melacak apakah ada error yang terjadi
    let mut any_error_occurred = false;

    // Loop melalui semua argumen mulai dari indeks 1 (melewati nama program)
    // &args[1..] membuat slice dari vektor args, berisi semua elemen dari indeks 1 sampai akhir.
    for filename in &args[1..] {
        // Panggil fungsi process_file untuk setiap nama file
        // Kita gunakan 'match' untuk menangani Result yang dikembalikan
        match process_file(filename) {
            // Jika sukses (Ok), tidak perlu melakukan apa-apa
            Ok(_) => {}
            // Jika error (Err), cetak pesan error spesifik ke stderr
            Err(e) => {
                // Pesan error yang baik mencantumkan nama program dan file yang bermasalah
                eprintln!("rcat: {}: {}", filename, e);
                // Set flag bahwa setidaknya satu error telah terjadi
                any_error_occurred = true;
            }
        }
    }

    // Setelah loop selesai, periksa apakah ada error yang terjadi
    if any_error_occurred {
        // Jika ya, keluar program dengan status 1 (menandakan error)
        process::exit(1);
    }
    // Jika tidak ada error sama sekali, program akan selesai secara normal
    // dengan status 0 (sukses) secara implisit.
}