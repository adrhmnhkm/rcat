mod cli;
mod clipboard;
mod config;
mod git;
mod hex;
mod image_view;
mod printer;
mod syntax;

use clap::Parser;
use cli::{Cli, StyleMode};
use printer::Printer;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process;
use syntect::easy::HighlightLines;

fn process_file(
    filename: &str,
    cli: &Cli,
    start_line: usize,
    start_numbered: usize,
) -> io::Result<(usize, usize)> {
    let path = Path::new(filename);

    // 0. Jika file adalah gambar atau diminta dengan --image
    if cli.image || (!cli.force && !cli.hex && image_view::is_image_file(path)) {
        let mut stdout = io::stdout().lock();
        let is_full = matches!(cli.effective_style(), StyleMode::Full | StyleMode::Header);
        if let Err(e) = image_view::print_image_preview(path, &mut stdout, 75, is_full) {
            eprintln!("rcat: Gagal menampilkan gambar '{}': {}", filename, e);
        }
        return Ok((start_line, start_numbered));
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // 1. Jika pengguna meminta Hex Dump secara eksplisit (-X, --hex)
    if cli.hex {
        let mut stdout = io::stdout().lock();
        let is_full = matches!(cli.effective_style(), StyleMode::Full | StyleMode::Header);
        hex::print_hex_dump(&mut reader, &mut stdout, Some(filename), is_full)?;
        return Ok((start_line, start_numbered));
    }

    // 2. Deteksi file biner (tanpa mengonsumsi buffer pembacaan)
    let sample = reader.fill_buf()?;
    if !cli.force && hex::is_binary_buffer(sample) {
        eprintln!(
            "rcat: '{}' terdeteksi sebagai file biner. Gunakan -X / --hex untuk melihat hex dump, atau -f / --force untuk memaksa teks.",
            filename
        );
        return Ok((start_line, start_numbered));
    }

    // Baca baris pertama untuk deteksi sintaks (shebang / header)
    let mut first_line_buf = String::new();
    let has_first_line = reader.read_line(&mut first_line_buf)? > 0;
    let first_line_str = if has_first_line {
        Some(first_line_buf.trim_end_matches(['\r', '\n']))
    } else {
        None
    };

    let syntax = syntax::resolve_syntax(cli.language.as_deref(), Some(path), first_line_str);
    let theme_name = cli.theme.as_deref().unwrap_or(syntax::DEFAULT_THEME);
    let theme = syntax::get_theme(theme_name);
    let highlighter = HighlightLines::new(syntax, theme);

    let (init_line, init_num) =
        if matches!(cli.effective_style(), StyleMode::Full | StyleMode::Header) {
            (0, 0)
        } else {
            (start_line, start_numbered)
        };

    let mut printer = Printer::with_counters(cli, highlighter, Some(filename), init_line, init_num);
    printer.set_syntax_name(Some(syntax.name.clone()));

    if cli.is_git_enabled() {
        let tracker = git::GitDiffTracker::for_file(path);
        printer.set_git_tracker(tracker);
    }
    let mut stdout = io::stdout().lock();

    if let Some(first_line) = first_line_str {
        printer.process_line(first_line, &mut stdout)?;
        let mut line_buf = String::new();
        while reader.read_line(&mut line_buf)? > 0 {
            let trimmed = line_buf.trim_end_matches(['\r', '\n']);
            printer.process_line(trimmed, &mut stdout)?;
            line_buf.clear();
        }
    }

    printer.print_footer(&mut stdout)?;
    Ok(printer.counters())
}

fn process_stdin(cli: &Cli) -> io::Result<()> {
    let stdin = io::stdin();
    let mut handle = BufReader::new(stdin.lock());

    // 1. Jika pengguna meminta Hex Dump secara eksplisit (-X, --hex)
    if cli.hex {
        let mut stdout = io::stdout().lock();
        let is_full = matches!(cli.effective_style(), StyleMode::Full | StyleMode::Header);
        hex::print_hex_dump(&mut handle, &mut stdout, Some("stdin"), is_full)?;
        return Ok(());
    }

    // 2. Deteksi input biner
    let sample = handle.fill_buf()?;
    if !cli.force && hex::is_binary_buffer(sample) {
        eprintln!(
            "rcat: Input stdin terdeteksi sebagai data biner. Gunakan -X / --hex untuk melihat hex dump, atau -f / --force untuk memaksa teks."
        );
        return Ok(());
    }

    // Baca baris pertama untuk deteksi sintaks
    let mut first_line_buf = String::new();
    let has_first_line = handle.read_line(&mut first_line_buf)? > 0;
    let first_line_str = if has_first_line {
        Some(first_line_buf.trim_end_matches(['\r', '\n']))
    } else {
        None
    };

    let syntax = syntax::resolve_syntax(cli.language.as_deref(), None, first_line_str);
    let theme_name = cli.theme.as_deref().unwrap_or(syntax::DEFAULT_THEME);
    let theme = syntax::get_theme(theme_name);
    let highlighter = HighlightLines::new(syntax, theme);

    let mut printer = Printer::new(cli, highlighter, Some("stdin"));
    printer.set_syntax_name(Some(syntax.name.clone()));
    let mut stdout = io::stdout().lock();

    if let Some(first_line) = first_line_str {
        printer.process_line(first_line, &mut stdout)?;
        let mut line_buf = String::new();
        while handle.read_line(&mut line_buf)? > 0 {
            let trimmed = line_buf.trim_end_matches(['\r', '\n']);
            printer.process_line(trimmed, &mut stdout)?;
            line_buf.clear();
        }
    }

    printer.print_footer(&mut stdout)?;
    Ok(())
}

fn main() {
    let mut cli = Cli::parse();

    // Muat konfigurasi pengguna jika ada
    let cfg = config::Config::load(cli.config.as_deref(), cli.no_config);
    cli.apply_config(&cfg);

    // 1. Tampilkan daftar tema jika diminta
    if cli.list_themes {
        syntax::print_themes();
        process::exit(0);
    }

    // 2. Tampilkan daftar bahasa jika diminta
    if cli.list_languages {
        syntax::print_languages();
        process::exit(0);
    }

    let mut any_error = false;

    // 3. Proses input dari stdin atau file
    if cli.files.is_empty() {
        if let Err(e) = process_stdin(&cli) {
            eprintln!("rcat: Gagal membaca standard input: {}", e);
            any_error = true;
        }
    } else {
        let mut current_line = 0;
        let mut current_numbered = 0;

        for (idx, filename) in cli.files.iter().enumerate() {
            // Berikan spasi pemisah antar file jika mode Full atau Header
            if idx > 0 && matches!(cli.effective_style(), StyleMode::Full | StyleMode::Header) {
                println!();
            }

            match process_file(filename, &cli, current_line, current_numbered) {
                Ok((next_line, next_num)) => {
                    current_line = next_line;
                    current_numbered = next_num;
                }
                Err(e) => {
                    eprintln!("rcat: {}: {}", filename, e);
                    any_error = true;
                }
            }
        }
    }

    if any_error {
        process::exit(1);
    }
}
