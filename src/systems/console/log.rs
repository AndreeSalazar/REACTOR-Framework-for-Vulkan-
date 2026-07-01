use crate::systems::console::color;

const W: usize = 64;

pub struct Log;

impl Log {
    pub fn engine(msg: &str) {
        println!(
            "{}{}⚙ REACTOR{}  {}",
            color::BOLD,
            color::CYAN,
            color::RESET,
            msg,
        );
    }

    pub fn success(msg: &str) {
        println!("{}  ✓{} {}", color::GREEN, color::RESET, msg,);
    }

    pub fn warn(msg: &str) {
        println!("{}  ⚠{} {}", color::YELLOW, color::RESET, msg,);
    }

    pub fn error(msg: &str) {
        println!("{}  ✗{} {}", color::RED, color::RESET, msg,);
    }

    pub fn game(msg: &str) {
        println!("{}🎮{} {}", color::MAGENTA, color::RESET, msg,);
    }

    pub fn audio(msg: &str) {
        println!("{}🔊{} {}", color::BLUE, color::RESET, msg,);
    }

    pub fn asset(msg: &str) {
        println!("{}📦{} {}", color::CYAN, color::RESET, msg,);
    }

    pub fn info(msg: &str) {
        println!("{}  · {}{}", color::DIM, msg, color::RESET,);
    }

    pub fn section(title: &str) {
        let prefix = format!("── {} ", title);
        let used = super::color::visual_width(&prefix);
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

    pub fn header(title: &str) {
        let inner = W - 2;
        let bar: String = std::iter::repeat('═').take(inner).collect();

        println!("{}{}╔{}╗{}", color::BOLD, color::CYAN, bar, color::RESET,);

        let title_vis = super::color::visual_width(title);
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

        println!("{}{}╚{}╝{}", color::BOLD, color::CYAN, bar, color::RESET,);
    }

    pub fn table(headers: &[&str], rows: &[Vec<String>], col_widths: &[usize]) {
        let border: String = col_widths
            .iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┬");
        println!("┌{}┐", border);

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

        let sep: String = col_widths
            .iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┼");
        println!("├{}┤", sep);

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

        let bot: String = col_widths
            .iter()
            .map(|&w| "─".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("┴");
        println!("└{}┘", bot);
    }

    pub fn kv(key: &str, value: &str) {
        let key_len = key.len();
        let val_len = value.len();
        let total_content = key_len + val_len + 4;
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
