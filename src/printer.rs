use crate::cli::{Cli, LineRange, StyleMode};
use crate::git::{GitDiffTracker, GitLineStatus};
use crate::syntax::SYNTAX_SET;
use std::io::{self, Write};
use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::util::as_24_bit_terminal_escaped;

const COLOR_GRAY: &str = "\x1b[90m";
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_CYAN_BOLD: &str = "\x1b[1;36m";
const COLOR_BLUE: &str = "\x1b[34m";

pub struct Printer<'a> {
    cli: &'a Cli,
    style_mode: StyleMode,
    highlighter: HighlightLines<'a>,
    file_label: Option<&'a str>,
    line_range: Option<LineRange>,
    line_counter: usize,
    numbered_counter: usize,
    prev_line_was_blank: bool,
    header_printed: bool,
    git_tracker: Option<GitDiffTracker>,
    highlight_regex: Option<regex::Regex>,
    copy_buffer: String,
    total_words: usize,
    total_chars: usize,
    syntax_name: Option<String>,
}

impl<'a> Printer<'a> {
    pub fn new(
        cli: &'a Cli,
        highlighter: HighlightLines<'a>,
        file_label: Option<&'a str>,
    ) -> Self {
        Self::with_counters(cli, highlighter, file_label, 0, 0)
    }

    pub fn with_counters(
        cli: &'a Cli,
        highlighter: HighlightLines<'a>,
        file_label: Option<&'a str>,
        initial_line: usize,
        initial_numbered: usize,
    ) -> Self {
        let style_mode = cli.effective_style();
        Self {
            cli,
            style_mode,
            highlighter,
            file_label,
            line_range: cli.line_range,
            line_counter: initial_line,
            numbered_counter: initial_numbered,
            prev_line_was_blank: false,
            header_printed: false,
            git_tracker: None,
            highlight_regex: cli.highlight.as_deref().and_then(|h| regex::Regex::new(h).ok()),
            copy_buffer: String::new(),
            total_words: 0,
            total_chars: 0,
            syntax_name: None,
        }
    }

    pub fn set_syntax_name(&mut self, name: Option<String>) {
        self.syntax_name = name;
    }

    pub fn set_git_tracker(&mut self, tracker: Option<GitDiffTracker>) {
        self.git_tracker = tracker;
    }

    pub fn counters(&self) -> (usize, usize) {
        (self.line_counter, self.numbered_counter)
    }

    /// Cetak header pembuka jika gaya tampilan memerlukannya
    pub fn print_header<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        if self.header_printed {
            return Ok(());
        }

        let label = self.file_label.unwrap_or("stdin");
        match self.style_mode {
            StyleMode::Full => {
                writeln!(
                    writer,
                    "{}───────┬──────────────────────────────────────────────────────────────────{}",
                    COLOR_GRAY, COLOR_RESET
                )?;
                writeln!(
                    writer,
                    "{}       │{} {}{}{}",
                    COLOR_GRAY, COLOR_RESET, COLOR_CYAN_BOLD, label, COLOR_RESET
                )?;
                writeln!(
                    writer,
                    "{}───────┼──────────────────────────────────────────────────────────────────{}",
                    COLOR_GRAY, COLOR_RESET
                )?;
            }
            StyleMode::Header => {
                writeln!(
                    writer,
                    "{}──────────────────────────────────────────────────────────────────────────{}",
                    COLOR_GRAY, COLOR_RESET
                )?;
                writeln!(
                    writer,
                    "{}{}{}",
                    COLOR_CYAN_BOLD, label, COLOR_RESET
                )?;
                writeln!(
                    writer,
                    "{}──────────────────────────────────────────────────────────────────────────{}",
                    COLOR_GRAY, COLOR_RESET
                )?;
            }
            _ => {}
        }

        self.header_printed = true;
        Ok(())
    }

    /// Cetak footer penutup jika gaya tampilan memerlukannya
    pub fn print_footer<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        if !self.header_printed {
            return Ok(());
        }

        // Tampilkan statistik jika diminta (--stats)
        if self.cli.stats {
            let label = self.file_label.unwrap_or("stdin");
            let lang = self.syntax_name.as_deref().unwrap_or("Plain Text");
            let stats_line = format!(
                " File: {} │ Baris: {} │ Kata: {} │ Karakter: {} │ Bahasa: {}",
                label, self.line_counter, self.total_words, self.total_chars, lang
            );
            match self.style_mode {
                StyleMode::Full => {
                    writeln!(
                        writer,
                        "{}───────┼──────────────────────────────────────────────────────────────────{}",
                        COLOR_GRAY, COLOR_RESET
                    )?;
                    writeln!(
                        writer,
                        "{}       │{} {}{}{}",
                        COLOR_GRAY, COLOR_RESET, COLOR_GRAY, stats_line, COLOR_RESET
                    )?;
                }
                StyleMode::Header => {
                    writeln!(
                        writer,
                        "{}──────────────────────────────────────────────────────────────────────────{}",
                        COLOR_GRAY, COLOR_RESET
                    )?;
                    writeln!(writer, "{}{}{}", COLOR_GRAY, stats_line, COLOR_RESET)?;
                }
                _ => {
                    writeln!(writer, "{}{}{}", COLOR_GRAY, stats_line, COLOR_RESET)?;
                }
            }
        }

        match self.style_mode {
            StyleMode::Full => {
                writeln!(
                    writer,
                    "{}───────┴──────────────────────────────────────────────────────────────────{}",
                    COLOR_GRAY, COLOR_RESET
                )?;
            }
            StyleMode::Header => {
                writeln!(
                    writer,
                    "{}──────────────────────────────────────────────────────────────────────────{}",
                    COLOR_GRAY, COLOR_RESET
                )?;
            }
            _ => {}
        }

        // Salin ke clipboard jika diminta (-c, --copy)
        if self.cli.copy && !self.copy_buffer.is_empty() {
            if let Err(e) = crate::clipboard::copy_to_clipboard(&self.copy_buffer) {
                eprintln!("rcat: Gagal menyalin ke clipboard: {}", e);
            } else {
                eprintln!(
                    "\x1b[32mrcat: ✓ Berhasil disalin ke clipboard ({} baris)\x1b[0m",
                    self.line_counter
                );
            }
        }

        Ok(())
    }

    /// Memproses dan mencetak satu baris teks
    pub fn process_line<W: Write>(&mut self, raw_line: &str, writer: &mut W) -> io::Result<()> {
        self.line_counter += 1;
        let line_num = self.line_counter;

        // Normalisasi akhir baris jika ada \r
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let is_blank = line.is_empty();

        // 1. Squeeze Blank Lines (-s, --squeeze-blank)
        if self.cli.squeeze_blank && is_blank && self.prev_line_was_blank {
            return Ok(());
        }
        self.prev_line_was_blank = is_blank;

        // Update internal state highlighter syntect (harus dipanggil untuk setiap baris
        // agar parser state tetap sinkron meskipun baris tidak ditampilkan di layar)
        let ranges: Vec<(Style, &str)> = self
            .highlighter
            .highlight_line(line, &SYNTAX_SET)
            .map_err(|e| io::Error::other(format!("Syntect error: {}", e)))?;

        // Hitung statistik kata dan karakter
        self.total_words += line.split_whitespace().count();
        self.total_chars += line.chars().count();

        // 2. Line Range Filter (-r, --line-range)
        if let Some(ref range) = self.line_range
            && !range.contains(line_num)
        {
            return Ok(());
        }

        if self.cli.copy {
            self.copy_buffer.push_str(line);
            self.copy_buffer.push('\n');
        }

        // Pastikan header dicetak sebelum baris pertama keluar
        self.print_header(writer)?;

        // 3. Modifikasi Karakter TAB (-T, -A)
        let mut formatted_content = if self.cli.display_tabs() {
            // Highlight per-range lalu gantikan \t dengan ^I
            let mut highlighted_line = as_24_bit_terminal_escaped(&ranges[..], true);
            if highlighted_line.contains('\t') {
                highlighted_line = highlighted_line.replace('\t', &format!("{}^I{}", COLOR_BLUE, COLOR_RESET));
            }
            highlighted_line
        } else {
            as_24_bit_terminal_escaped(&ranges[..], true)
        };

        // 3b. Keyword / Regex Highlighting (-H, --highlight)
        if let Some(ref re) = self.highlight_regex {
            formatted_content = re.replace_all(&formatted_content, "\x1b[1;7m$0\x1b[27;22m").to_string();
        }

        // 4. Modifikasi Akhir Baris (-E, -A)
        let end_marker = if self.cli.display_ends() {
            format!("{}${}", COLOR_BLUE, COLOR_RESET)
        } else {
            String::new()
        };

        // 5. Penomoran Baris (-n, -b, style)
        self.render_line_with_style(writer, line_num, is_blank, &formatted_content, &end_marker)?;

        Ok(())
    }

    fn render_line_with_style<W: Write>(
        &mut self,
        writer: &mut W,
        line_num: usize,
        is_blank: bool,
        content: &str,
        end_marker: &str,
    ) -> io::Result<()> {
        let display_number_str = if self.cli.number_nonblank {
            if is_blank {
                String::new()
            } else {
                self.numbered_counter += 1;
                format!("{:>6}", self.numbered_counter)
            }
        } else if self.cli.should_number() {
            format!("{:>6}", line_num)
        } else {
            String::new()
        };

        let git_marker = if let Some(ref tracker) = self.git_tracker {
            match tracker.get_status(line_num) {
                Some(GitLineStatus::Added) => "\x1b[32m+\x1b[0m",
                Some(GitLineStatus::Modified) => "\x1b[33m~\x1b[0m",
                Some(GitLineStatus::Removed) => "\x1b[31m-\x1b[0m",
                None => " ",
            }
        } else {
            " "
        };

        match self.style_mode {
            StyleMode::Full | StyleMode::Grid => {
                let num_display = if display_number_str.is_empty() {
                    "      ".to_string()
                } else {
                    display_number_str
                };
                writeln!(
                    writer,
                    "{}{}{}{}{}│{} {}{}",
                    COLOR_GRAY, num_display, COLOR_RESET, git_marker, COLOR_GRAY, COLOR_RESET, content, end_marker
                )
            }
            StyleMode::Numbers => {
                if display_number_str.is_empty() {
                    writeln!(writer, "{}{}", content, end_marker)
                } else {
                    writeln!(
                        writer,
                        "{}{}{}\t{}{}",
                        COLOR_GRAY, display_number_str, COLOR_RESET, content, end_marker
                    )
                }
            }
            StyleMode::Plain | StyleMode::Header => {
                if !display_number_str.is_empty() {
                    writeln!(
                        writer,
                        "{}{}{}\t{}{}",
                        COLOR_GRAY, display_number_str, COLOR_RESET, content, end_marker
                    )
                } else {
                    writeln!(writer, "{}{}", content, end_marker)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{get_theme, DEFAULT_THEME, SYNTAX_SET};

    #[test]
    fn test_printer_squeeze_and_range() {
        let cli = Cli {
            number: true,
            number_nonblank: false,
            squeeze_blank: true,
            show_ends: true,
            show_tabs: true,
            show_all: false,
            language: None,
            line_range: Some(LineRange { start: 2, end: 4 }),
            style: Some(StyleMode::Numbers),
            theme: None,
            list_themes: false,
            list_languages: false,
            hex: false,
            force: false,
            config: None,
            no_config: false,
            git: false,
            no_git: false,
            highlight: None,
            copy: false,
            stats: false,
            image: false,
            files: vec![],
        };

        let syntax = SYNTAX_SET.find_syntax_plain_text();
        let theme = get_theme(DEFAULT_THEME);
        let highlighter = HighlightLines::new(syntax, theme);

        let mut printer = Printer::new(&cli, highlighter, Some("test.txt"));
        let mut output = Vec::new();

        let lines = vec![
            "Baris 1",
            "Baris 2 dengan \ttab",
            "",
            "",
            "Baris 5",
        ];

        for l in lines {
            printer.process_line(l, &mut output).unwrap();
        }
        printer.print_footer(&mut output).unwrap();

        let res = String::from_utf8_lossy(&output);
        // Baris 1 tidak boleh ada karena range 2..4
        assert!(!res.contains("Baris 1"));
        // Baris 2 harus ada
        assert!(res.contains("Baris 2"));
        // Tab harus diganti ^I
        assert!(res.contains("^I"));
        // End marker $ harus ada
        assert!(res.contains('$'));
        // Baris 5 tidak boleh ada karena range berakhir di 4
        assert!(!res.contains("Baris 5"));
    }
}
