// module with various logging and printing utilities for internal use
use console::Style;

pub fn print_success(message: &str) {
    let style = Style::new().green().bold();
    println!("{}", style.apply_to(message));
}

pub fn print_error(message: &str) {
    let style = Style::new().red().bold();
    eprintln!("{}", style.apply_to(message));
}

pub fn print_info(message: &str) {
    let style = Style::new().italic();
    println!("{}", style.apply_to(message));
}
