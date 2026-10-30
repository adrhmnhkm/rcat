use once_cell::sync::Lazy;
use std::path::Path;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

pub const DEFAULT_THEME: &str = "base16-ocean.dark";

/// Mengambil Theme berdasarkan nama, atau fallback ke tema default
pub fn get_theme(theme_name: &str) -> &'static Theme {
    THEME_SET.themes.get(theme_name).unwrap_or_else(|| {
        eprintln!(
            "rcat: Peringatan: Tema '{}' tidak ditemukan. Menggunakan default '{}'.",
            theme_name, DEFAULT_THEME
        );
        &THEME_SET.themes[DEFAULT_THEME]
    })
}

/// Menentukan sintaks berdasarkan prioritas:
/// 1. Flag bahasa manual (--language)
/// 2. Ekstensi file
/// 3. Baris pertama file (shebang / header)
/// 4. Plain text fallback
pub fn resolve_syntax(
    language_override: Option<&str>,
    file_path: Option<&Path>,
    first_line: Option<&str>,
) -> &'static SyntaxReference {
    if let Some(lang) = language_override {
        if let Some(syntax) = SYNTAX_SET
            .find_syntax_by_token(lang)
            .or_else(|| SYNTAX_SET.find_syntax_by_name(lang))
            .or_else(|| SYNTAX_SET.find_syntax_by_extension(lang))
        {
            return syntax;
        }

        eprintln!(
            "rcat: Peringatan: Bahasa '{}' tidak ditemukan di database syntect. Mencoba auto-detect...",
            lang
        );
    }

    // Coba tebak dari ekstensi file
    if let Some(path) = file_path
        && let Some(ext) = path.extension().and_then(|s| s.to_str())
        && let Some(syntax) = SYNTAX_SET.find_syntax_by_extension(ext)
    {
        return syntax;
    }

    // Coba tebak dari baris pertama (misal: #!/bin/bash, #!/usr/bin/env python)
    if let Some(line) = first_line
        && let Some(syntax) = SYNTAX_SET.find_syntax_by_first_line(line)
    {
        return syntax;
    }

    // Fallback terakhir: Plain text
    SYNTAX_SET.find_syntax_plain_text()
}

/// Cetak semua tema yang tersedia
pub fn print_themes() {
    println!("Daftar tema yang tersedia:");
    let mut names: Vec<_> = THEME_SET.themes.keys().collect();
    names.sort();
    for name in names {
        if name == DEFAULT_THEME {
            println!("- {} (default)", name);
        } else {
            println!("- {}", name);
        }
    }
}

/// Cetak semua bahasa sintaks yang didukung
pub fn print_languages() {
    println!("Daftar bahasa sintaks yang didukung:");
    let mut syntaxes: Vec<_> = SYNTAX_SET.syntaxes().iter().collect();
    syntaxes.sort_by_key(|s| &s.name);
    for s in syntaxes {
        let exts = if s.file_extensions.is_empty() {
            String::new()
        } else {
            format!(" [{}]", s.file_extensions.join(", "))
        };
        println!("- {}{}", s.name, exts);
    }
}
