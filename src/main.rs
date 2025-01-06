use notify::{Event, RecursiveMode, Result as NResult, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::{
    env, fs,
    io::{stdout, Write},
    process::{self, Command},
};
use termion::{clear, color, cursor, style};

fn render_result(filepath: &String) {
    let filename = filepath.split("/").last().unwrap_or("");
    let command = format!(r#"echo '#use "{}";;' | ocaml"#, filepath);
    let result = Command::new("bash").arg("-c").arg(command).output();

    let mut stdout = stdout();

    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();

    let mut write_error = |message: &str| {
        write!(
            stdout,
            "{}{}{}error{}: {}",
            cursor::Goto(1, 1),
            color::Fg(color::Red),
            style::Bold,
            style::Reset,
            message
        )
        .unwrap();
    };

    // print execution errors
    if result.is_err() {
        write_error(format!("Could not execute the file `{}`", filename).as_str());
        return;
    }

    let output = result.as_ref().ok().unwrap();
    if !output.status.success() {
        write_error(format!("An error occured while executing `{}`", filename).as_str());
        return;
    }

    // print successful execution
    write!(
        stdout,
        "{}{}{}success{}: Successfully executed the file `{}`:",
        cursor::Goto(1, 1),
        color::Fg(color::Green),
        style::Bold,
        style::Reset,
        filename
    )
    .unwrap();

    let output_text = String::from_utf8(output.clone().stdout).unwrap();
    write!(stdout, "{}{}", cursor::Goto(1, 3), output_text).unwrap();

    return;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // transform path to an absolute address if possible
    let filepath = match args.get(1) {
        None => None,
        Some(filepath) if !filepath.ends_with(".ml") => None,
        Some(filepath) if fs::metadata(filepath).is_err() => None,

        Some(filepath) => match fs::canonicalize(filepath) {
            Ok(absolute_path) => Some(absolute_path.display().to_string()),
            Err(_) => None,
        },
    };

    if filepath.is_none() {
        eprintln!("Error: Bad file path provided");
        process::exit(1);
    }

    let filepath = filepath.unwrap();
    // check for live mode enabled
    let live_mode = args.get(2).map_or(false, |arg| arg == "--live");

    // render result
    render_result(&filepath);

    // enter live reload mode
    if !live_mode {
        process::exit(0)
    }

    let (tx, rx) = mpsc::channel::<NResult<Event>>();
    let mut watcher = notify::recommended_watcher(tx).unwrap();

    watcher
        .watch(Path::new(filepath.as_str()), RecursiveMode::NonRecursive)
        .unwrap();

    for res in rx {
        match res {
            Ok(res) if res.kind.is_modify() => render_result(&filepath),
            _ => (),
        }
    }
}
