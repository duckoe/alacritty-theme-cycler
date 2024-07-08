use std::io::Cursor;

use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim,
};

#[derive(Debug, serde::Deserialize)]
struct Theme {
    import: Vec<String>,
}

fn get_current_theme() -> String {
    // read ~/.config/alacritty/alacritty.toml
    // and get the current theme from import key
    let home_dir = std::env::var("HOME").unwrap();
    let alac_path = format!("{}/.config/alacritty/alacritty.toml", home_dir);
    let alacritty_conf_str = std::fs::read_to_string(alac_path).unwrap();
    let t: Theme = toml::from_str(alacritty_conf_str.as_str()).unwrap();
    if t.import.len() == 0 {
        panic!("No theme found");
    }
    let theme_file = t.import.last().unwrap();
    let theme_name = theme_file.split("/").last().unwrap().replace(".toml", "");
    theme_name
}

fn get_theme_list() -> Vec<String> {
    let output = std::process::Command::new("alacritty-theme-switcher")
        .arg("-l")
        .output()
        .expect("failed to execute process");

    let themes = String::from_utf8_lossy(&output.stdout)
        .split("\n")
        .map(|x| x.trim().to_string())
        .collect::<Vec<String>>()
        .clone();
    themes
}

fn get_cur_idx() -> usize {
    let themes = get_theme_list();
    let cur_theme = get_current_theme();
    themes
        .iter()
        .position(|x| x.as_str() == cur_theme.as_str())
        .unwrap()
}

fn green(s: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", s)
}
fn bold(s: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", s)
}
fn switch_theme(theme: &str) {
    let output = std::process::Command::new("alacritty-theme-switcher")
        .arg(theme)
        .output()
        .expect("failed to change theme");
    println!(
        "↪️ Changed alacritty theme to: {}",
        bold(green(theme).as_str())
    );
}

fn fzf_select_theme(themes: Vec<String>) -> String {
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .build()
        .unwrap();

    let input = themes.join("\n");

    // `SkimItemReader` is a helper to turn any `BufRead` into a stream of `SkimItem`
    // `SkimItem` was implemented for `AsRef<str>` by default
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    // `run_with` would read and show items from the stream
    let selected_items = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    selected_items.last().unwrap().output().to_string()
}

fn main() {
    let cur_theme = get_current_theme();
    let themes = get_theme_list();
    // execute shell command
    let cur_idx = get_cur_idx();

    // get arg (n or p)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let arg = args.get(1).unwrap();
        if arg == "n" {
            let next_idx = (cur_idx + 1) % themes.len();
            let next_theme = themes.get(next_idx).unwrap();

            switch_theme(next_theme);
            return;
        }
        if arg == "p" {
            let prev_idx = (cur_idx + themes.len() - 1) % themes.len();
            let prev_theme = themes.get(prev_idx).unwrap();
            switch_theme(prev_theme);
            return;
        }

        if arg == "c" {
            println!("{}", cur_theme);
            return;
        } else if arg == "l" {
            for theme in themes {
                println!("{}", theme);
            }
            return;
        } else if arg == "f" {
            let selected = fzf_select_theme(themes);
            switch_theme(selected.as_str());
            //fzf
        } else {
            let theme = args.get(1).unwrap();
            switch_theme(theme);
            return;
        }
    } else {
        let selected = fzf_select_theme(themes);
        switch_theme(selected.as_str());
    }
}
