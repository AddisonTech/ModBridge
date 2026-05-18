use chrono::Local;
use colored::Colorize;

const COL_REG: usize = 12;
const COL_DEC: usize = 14;
const COL_HEX: usize = 12;

fn header_line() -> String {
    format!(
        "{:>COL_REG$}  {:>COL_DEC$}  {:>COL_HEX$}",
        "Register".bold().cyan(),
        "Value (dec)".bold().cyan(),
        "Value (hex)".bold().cyan(),
    )
}

fn separator() -> String {
    format!(
        "{:->COL_REG$}  {:->COL_DEC$}  {:->COL_HEX$}",
        "", "", ""
    )
    .dimmed()
    .to_string()
}

pub fn print_table(values: &[(u16, u16)]) {
    // Move cursor up to overwrite previous table if not the first render
    let ts = Local::now().format("%Y-%m-%d  %H:%M:%S");
    println!(
        "\n  {}  {}",
        "ModBridge".bold().cyan(),
        format!("{ts}").dimmed()
    );
    println!("  {}", separator());
    println!("  {}", header_line());
    println!("  {}", separator());
    for (addr, val) in values {
        println!(
            "  {:>COL_REG$}  {:>COL_DEC$}  {:>COL_HEX$}",
            addr.to_string().bold(),
            val.to_string(),
            format!("0x{val:04X}").dimmed(),
        );
    }
    println!("  {}", separator());
}
