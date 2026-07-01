use crate::systems::console::color;

const W: usize = 64;

pub struct ReactorBanner;

impl ReactorBanner {
    pub fn print_init(
        title: &str,
        resolution: &str,
        msaa: &str,
        ray_tracing: bool,
        gpu_name: &str,
    ) {
        let bc = format!("{}{}", color::DIM, color::CYAN);
        let tc = format!("{}{}", color::BOLD, color::BRIGHT_CYAN);
        let r = color::RESET;
        let inner = W - 2;
        let bar: String = std::iter::repeat('═').take(inner).collect();

        println!("{}╔{}╗{}", bc, bar, r);
        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        let art: &[&str] = &[
            "██████  ███████  █████   ██████ ████████  ██████  ██████ ",
            "██   ██ ██      ██   ██ ██         ██    ██    ██ ██   ██",
            "██████  █████   ███████ ██         ██    ██    ██ ██████ ",
            "██   ██ ██      ██   ██ ██         ██    ██    ██ ██   ██",
            "██   ██ ███████ ██   ██  ██████    ██     ██████  ██   ██",
        ];

        for line in art {
            let vis = super::color::visual_width(line);
            let pad = if inner > vis + 3 { inner - vis - 3 } else { 0 };
            println!(
                "{}║   {}{}{}{}{}║{}",
                bc, tc, line, r, " ".repeat(pad), bc, r,
            );
        }

        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        let version = format!("Vulkan Engine v{}", env!("CARGO_PKG_VERSION"));
        let vis = super::color::visual_width(&version);
        let pad = if inner > vis + 3 { inner - vis - 3 } else { 0 };
        println!(
            "{}║   {}{}{}{}{}║{}",
            bc, color::DIM, color::WHITE, version, " ".repeat(pad), bc, r,
        );

        println!("{}╠{}╣{}", bc, bar, r);

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
        {
            let label_str = format!("{}Ray Tracing:{} {}", color::CYAN, r, rt_label,);
            let vis = super::color::visual_width_of_plain("Ray Tracing: ")
                + super::color::visual_width_of_plain(if ray_tracing {
                    "✅ Enabled"
                } else {
                    "❌ Disabled"
                });
            let pad = if inner > vis + 2 { inner - vis - 2 } else { 0 };
            println!("{}║  {}{}{}║{}", bc, label_str, " ".repeat(pad), bc, r,);
        }

        println!("{}╚{}╝{}", bc, bar, r);
    }

    fn print_detail_row(bc: &str, inner: usize, key: &str, value: &str, r: &str) {
        let plain_len = key.len() + 2 + value.len();
        let pad = if inner > plain_len + 2 {
            inner - plain_len - 2
        } else {
            0
        };
        println!(
            "{}║  {}{}: {}{}{} {}{}║{}",
            bc, color::CYAN, key, color::BRIGHT_WHITE, value, r, " ".repeat(pad), bc, r,
        );
    }
}

pub struct GameBanner;

impl GameBanner {
    pub fn print(lines: &[&str], subtitle: &str, col: &str) {
        let bc = format!("{}{}", color::DIM, color::CYAN);
        let r = color::RESET;
        let inner = W - 2;
        let bar: String = std::iter::repeat('═').take(inner).collect();

        println!("{}╔{}╗{}", bc, bar, r);
        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        for line in lines {
            let vis = super::color::visual_width(line);
            let pad = if inner > vis + 3 { inner - vis - 3 } else { 0 };
            println!(
                "{}║   {}{}{}{}{}║{}",
                bc, col, line, r, " ".repeat(pad), bc, r,
            );
        }

        println!("{}║{}{}║{}", bc, " ".repeat(inner), bc, r);

        let sub_vis = super::color::visual_width(subtitle);
        let sub_pad = if inner > sub_vis + 3 {
            inner - sub_vis - 3
        } else {
            0
        };
        println!(
            "{}║   {}{}{}{}{}║{}",
            bc, color::DIM, subtitle, r, " ".repeat(sub_pad), bc, r,
        );

        println!("{}╚{}╝{}", bc, bar, r);
    }
}
