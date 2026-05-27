//! # REACTOR Console — Terminal Styling & Logging
//!
//! Provides colorful, structured terminal output for the REACTOR Vulkan engine.
//! Includes ANSI color constants, categorized log methods with Unicode prefixes,
//! box-drawing banners, tables, progress bars, and key-value formatting.
//!
//! On Windows 10+, call [`init()`] once at startup to enable
//! `ENABLE_VIRTUAL_TERMINAL_PROCESSING` so ANSI escape codes render correctly.
//!
//! This module is **self-contained** and depends only on `std`.

#![allow(dead_code)]

// ─── Default box width for banners ──────────────────────────────────────────
const W: usize = 64;

// ═══════════════════════════════════════════════════════════════════════════════
//  Windows ANSI initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Enable ANSI escape-code processing on Windows 10+ consoles.
///
/// This is a no-op on non-Windows targets.  Call once, early in `main()`.
#[cfg(windows)]
pub fn init() {
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

    extern "system" {
        fn GetStdHandle(nStdHandle: u32) -> isize;
        fn GetConsoleMode(hConsoleHandle: isize, lpMode: *mut u32) -> i32;
        fn SetConsoleMode(hConsoleHandle: isize, dwMode: u32) -> i32;
    }

    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle == -1 {
            return;
        }
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            return;
        }
        SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
    }
}

#[cfg(not(windows))]
pub fn init() {
    // ANSI is natively supported — nothing to do.
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Color constants & helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// ANSI color/style escape-code constants and RGB helpers.
pub mod color {
    // ── Styles ──────────────────────────────────────────────────────────────
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";

    // ── Foreground (standard) ───────────────────────────────────────────────
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    // ── Foreground (bright) ─────────────────────────────────────────────────
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";

    // ── Background ──────────────────────────────────────────────────────────
    pub const BG_RED: &str = "\x1b[41m";
    pub const BG_GREEN: &str = "\x1b[42m";
    pub const BG_BLUE: &str = "\x1b[44m";
    pub const BG_MAGENTA: &str = "\x1b[45m";

    /// Build a 24-bit true-color **foreground** escape sequence.
    ///
    /// ```ignore
    /// let orange = color::rgb(255, 165, 0);
    /// println!("{}orange text{}", orange, color::RESET);
    /// ```
    pub fn rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    }

    /// Build a 24-bit true-color **background** escape sequence.
    pub fn bg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Structured logging
// ═══════════════════════════════════════════════════════════════════════════════

/// Zero-sized type providing static, categorised log helpers.
///
/// Every method writes a single `println!` line with a coloured prefix.
pub struct Log;

impl Log {
    // ── Categorised one-liners ──────────────────────────────────────────────

    /// Engine-level message: `⚙ REACTOR  msg`
    pub fn engine(msg: &str) {
        println!(
            "{}{}⚙ REACTOR{}  {}",
            color::BOLD,
            color::CYAN,
            color::RESET,
            msg,
        );
    }

    /// Success: `  ✓ msg`
    pub fn success(msg: &str) {
        println!(
            "{}  ✓{} {}",
            color::GREEN,
            color::RESET,
            msg,
        );
    }

    /// Warning: `  ⚠ msg`
    pub fn warn(msg: &str) {
        println!(
            "{}  ⚠{} {}",
            color::YELLOW,
            color::RESET,
            msg,
        );
    }

    /// Error: `  ✗ msg`
    pub fn error(msg: &str) {
        println!(
            "{}  ✗{} {}",
            color::RED,
            color::RESET,
            msg,
        );
    }

    /// Game subsystem: `🎮 msg`
    pub fn game(msg: &str) {
        println!(
            "{}🎮{} {}",
            color::MAGENTA,
            color::RESET,
            msg,
        );
    }

    /// Audio subsystem: `🔊 msg`
    pub fn audio(msg: &str) {
        println!(
            "{}🔊{} {}",
            color::BLUE,
            color::RESET,
            msg,
        );
    }

    /// Asset pipeline: `📦 msg`
    pub fn asset(msg: &str) {
        println!(
            "{}📦{} {}",
            color::CYAN,
            color::RESET,
            msg,
        );
    }

    /// Informational detail: `  · msg`
    pub fn info(msg: &str) {
        println!(
            "{}  · {}{}",
            color::DIM,
            msg,
            color::RESET,
        );
    }

    // ── Section separator ───────────────────────────────────────────────────

    /// Print a section heading with a horizontal rule.
    ///
    /// ```text
    /// ── Title ──────────────────────────────────────────
    /// ```
    pub fn section(title: &str) {
        // "── " + title + " " + remaining dashes to fill W chars
        let prefix = format!("── {} ", title);
        let used = visual_width(&prefix);
        let remaining = if W > used { W - used } else { 0 };
        let tail: String = std::iter::repeat('─').take(remaining).collect();
        println!(
            "{}{}{}{}{}",
            color::BOLD,
            color::CYAN,
            prefix,
            tail,
            color::RESET,
        );
    }

    // ── Boxed header ────────────────────────────────────────────────────────

    /// Print a box-drawing header around `title`.
    ///
    /// ```text
    /// ╔══════════════════════════════════════╗
    /// ║  Title                              ║
    /// ╚══════════════════════════════════════╝
    /// ```
    pub fn header(title: &str) {
        let inner = W - 2; // space between the two vertical bars
        let bar: String = std::iter::repeat('═').take(inner).collect();

        println!(
            "{}{}╔{}╗{}",
            color::BOLD,
            color::CYAN,
            bar,
            color::RESET,
        );

        let title_vis = visual_width(title);
        let padding = if inner > title_vis + 2 {
            inner - title_vis - 2
        } else {
            0
        };
        println!(
            "{}{}║  {}{}{}{}║{}",
            color::BOLD,
            color::CYAN,
            color::BRIGHT_WHITE,
            title,
            " ".repeat(padding),
            color::CYAN,
            color::RESET,
        );

        println!(
            "{}{}╚{}╝{}",
            color::BOLD,
            color::CYAN,
            bar,
            color::RESET,
        );
    }

    // ── Table ───────────────────────────────────────────────────────────────

    /// Print a formatted table with box-drawing characters.
    ///
    /// * `headers`    – column names (rendered **bold + cyan**).
    /// * `rows`       – data rows.
    /// * `col_widths` – desired display width per column.
    pub fn table(headers: &[&str], rows: &[Vec<String>], col_widths: &[usize]) {
        // Top border
        let border: String = col_widths
            .iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┬");
        println!("┌{}┐", border);

        // Header row
        let hdr: String = headers
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let w = col_widths.get(i).copied().unwrap_or(10);
                format!(
                    " {}{}{}{:<pad$}{} ",
                    color::BOLD,
                    color::CYAN,
                    h,
                    "",
                    color::RESET,
                    pad = w.saturating_sub(h.len()),
                )
            })
            .collect::<Vec<_>>()
            .join("│");
        println!("│{}│", hdr);

        // Separator
        let sep: String = col_widths
            .iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┼");
        println!("├{}┤", sep);

        // Data rows
        for row in rows {
            let line: String = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let w = col_widths.get(i).copied().unwrap_or(10);
                    format!(" {:<pad$} ", cell, pad = w)
                })
                .collect::<Vec<_>>()
                .join("│");
            println!("│{}│", line);
        }

        // Bottom border
        let bot: String = col_widths
            .iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┴");
        println!("└{}┘", bot);
    }

    // ── Key-value pair ──────────────────────────────────────────────────────

    /// Print a dot-leader key-value pair.
    ///
    /// ```text
    ///   key ··········· value
    /// ```
    pub fn kv(key: &str, value: &str) {
        let key_len = key.len();
        let val_len = value.len();
        let total_content = key_len + val_len + 4; // 2 leading spaces + 1 space before dots + 1 space after dots
        let dots = if W > total_content {
            W - total_content
        } else {
            3
        };
        let dot_str: String = std::iter::repeat('·').take(dots).collect();
        println!(
            "  {}{}{} {}{}{} {}{}{}",
            color::CYAN,
            key,
            color::RESET,
            color::DIM,
            dot_str,
            color::RESET,
            color::WHITE,
            value,
            color::RESET,
        );
    }

    // ── Progress bar ────────────────────────────────────────────────────────

    /// Print a labelled progress bar.
    ///
    /// ```text
    ///   label [████████░░░░] 66%
    /// ```
    pub fn progress(label: &str, current: u32, max: u32) {
        let pct = if max == 0 {
            0
        } else {
            ((current as u64 * 100) / max as u64) as u32
        };

        let bar_width: u32 = 24;
        let filled = ((pct as u64 * bar_width as u64) / 100) as u32;
        let empty = bar_width - filled;

        let filled_str: String = std::iter::repeat('█').take(filled as usize).collect();
        let empty_str: String = std::iter::repeat('░').take(empty as usize).collect();

        println!(
            "  {} [{}{}{}{}{}] {}%",
            label,
            color::GREEN,
            filled_str,
            color::DIM,
            empty_str,
            color::RESET,
            pct,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  REACTOR engine banner
// ═══════════════════════════════════════════════════════════════════════════════

/// REACTOR ASCII-art initialization banner.
pub struct ReactorBanner;

impl ReactorBanner {
    /// Print the full REACTOR Framework startup banner.
    ///
    /// Shows block-letter ASCII art, engine version, and GPU / window details.
    pub fn print_init(
        title: &str,
        resolution: &str,
        msaa: &str,
        ray_tracing: bool,
        gpu_name: &str,
    ) {
        let bc = format!("{}{}", color::DIM, color::CYAN);       // border colour
        let tc = format!("{}{}", color::BOLD, color::BRIGHT_CYAN); // title colour
        let r = color::RESET;
        let inner = W - 2;
        let bar: String = std::iter::repeat('═').take(inner).collect();

        // ── Top border ──────────────────────────────────────────────────────
        println!("{}╔{}╗{}", bc, bar, r);

        // ── Empty spacer line ───────────────────────────────────────────────
        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        // ── ASCII art ───────────────────────────────────────────────────────
        let art: &[&str] = &[
            "██████  ███████  █████   ██████ ████████  ██████  ██████ ",
            "██   ██ ██      ██   ██ ██         ██    ██    ██ ██   ██",
            "██████  █████   ███████ ██         ██    ██    ██ ██████ ",
            "██   ██ ██      ██   ██ ██         ██    ██    ██ ██   ██",
            "██   ██ ███████ ██   ██  ██████    ██     ██████  ██   ██",
        ];

        for line in art {
            let vis = visual_width(line);
            let pad = if inner > vis + 3 { inner - vis - 3 } else { 0 };
            println!(
                "{}║   {}{}{}{}{}║{}",
                bc, tc, line, r, " ".repeat(pad), bc, r,
            );
        }

        // ── Empty spacer line ───────────────────────────────────────────────
        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        // ── Version tagline ─────────────────────────────────────────────────
        let version = "Vulkan Engine v1.2.0";
        let vis = visual_width(version);
        let pad = if inner > vis + 3 { inner - vis - 3 } else { 0 };
        println!(
            "{}║   {}{}{}{}{}║{}",
            bc,
            color::DIM,
            color::WHITE,
            version,
            " ".repeat(pad),
            bc,
            r,
        );

        // ── Mid separator ───────────────────────────────────────────────────
        println!("{}╠{}╣{}", bc, bar, r);

        // ── Detail rows ─────────────────────────────────────────────────────
        let rt_label = if ray_tracing {
            format!("{}✅ Enabled{}", color::GREEN, r)
        } else {
            format!("{}❌ Disabled{}", color::RED, r)
        };

        let details: &[(&str, &str)] = &[
            ("GPU", gpu_name),
            ("Title", title),
            ("Resolution", resolution),
            ("MSAA", msaa),
        ];

        for &(key, value) in details {
            Self::print_detail_row(&bc, inner, key, value, r);
        }
        // Ray Tracing row (value already has its own colour)
        {
            let label_str = format!(
                "{}Ray Tracing:{} {}",
                color::CYAN, r, rt_label,
            );
            let vis = visual_width_of_plain("Ray Tracing: ") + visual_width_of_plain(if ray_tracing { "✅ Enabled" } else { "❌ Disabled" });
            let pad = if inner > vis + 2 { inner - vis - 2 } else { 0 };
            println!(
                "{}║  {}{}{}║{}",
                bc, label_str, " ".repeat(pad), bc, r,
            );
        }

        // ── Bottom border ───────────────────────────────────────────────────
        println!("{}╚{}╝{}", bc, bar, r);
    }

    // helper: print a single "║  Key: Value                ║" row
    fn print_detail_row(bc: &str, inner: usize, key: &str, value: &str, r: &str) {
        let plain_len = key.len() + 2 + value.len(); // "Key: Value"
        let pad = if inner > plain_len + 2 {
            inner - plain_len - 2
        } else {
            0
        };
        println!(
            "{}║  {}{}: {}{}{} {}{}║{}",
            bc,
            color::CYAN,
            key,
            color::BRIGHT_WHITE,
            value,
            r,
            " ".repeat(pad),
            bc,
            r,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Game-specific banner
// ═══════════════════════════════════════════════════════════════════════════════

/// A banner for game-specific ASCII art titles.
pub struct GameBanner;

impl GameBanner {
    /// Print a framed game banner.
    ///
    /// * `lines`    – rows of ASCII art for the game title.
    /// * `subtitle` – tagline printed below the art.
    /// * `color`    – ANSI colour escape to use for the art text.
    pub fn print(lines: &[&str], subtitle: &str, col: &str) {
        let bc = format!("{}{}", color::DIM, color::CYAN);
        let r = color::RESET;
        let inner = W - 2;
        let bar: String = std::iter::repeat('═').take(inner).collect();

        println!("{}╔{}╗{}", bc, bar, r);
        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        for line in lines {
            let vis = visual_width(line);
            let pad = if inner > vis + 3 { inner - vis - 3 } else { 0 };
            println!(
                "{}║   {}{}{}{}{}║{}",
                bc, col, line, r, " ".repeat(pad), bc, r,
            );
        }

        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        // subtitle
        let sub_vis = visual_width(subtitle);
        let sub_pad = if inner > sub_vis + 3 { inner - sub_vis - 3 } else { 0 };
        println!(
            "{}║   {}{}{}{}{}║{}",
            bc,
            color::DIM,
            subtitle,
            r,
            " ".repeat(sub_pad),
            bc,
            r,
        );

        println!("{}╚{}╝{}", bc, bar, r);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Internal helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Approximate visual width of a string that may contain ANSI escapes.
///
/// Strips `\x1b[…m` sequences before counting characters.  This is a
/// best-effort heuristic that also counts multi-byte Unicode codepoints
/// (emoji, box-drawing) as one column each — good enough for padding.
fn visual_width(s: &str) -> usize {
    let mut width = 0usize;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        width += 1;
    }
    width
}

/// Visual width of a plain (no-ANSI) string — just `chars().count()`.
fn visual_width_of_plain(s: &str) -> usize {
    s.chars().count()
}

/// Helper para consultar el nombre corto de la GPU a partir del VulkanContext
pub fn gpu_name_short(context: &crate::core::context::VulkanContext) -> String {
    let instance = context.instance.get();
    let properties = unsafe { instance.get_physical_device_properties(context.physical_device) };

    let name_bytes: Vec<u8> = properties.device_name.iter()
        .map(|&c| c as u8)
        .take_while(|&b| b != 0)
        .collect();

    String::from_utf8_lossy(&name_bytes).into_owned()
}
