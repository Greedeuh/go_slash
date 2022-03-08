use std::{env, process::Command};

fn main() {
    let args: Vec<_> = env::args().collect();
    let shortcut: String = args.get(1).cloned().unwrap_or_else(|| "".to_owned());
    Command::new("xdg-open")
        .args([format!("http://go/{}", shortcut)])
        .output()
        .unwrap();
}
