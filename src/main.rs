use clap::{App, Arg, SubCommand};
use code_dup_detect::mark_dup_lines;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::Write;

fn main() {
    let matches = App::new("duplication-highlight")
        .about("Highlighter of duplicate code lines.")
        .version("1.0")
        .author("P. Horban <extremegf@gmail.com>")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to analyze")
                .required(true)
                .index(1),
        )
        .get_matches();
    let text = read_to_string(matches.value_of("INPUT").unwrap()).unwrap();
    let html = mark_dup_lines(&text);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("/tmp/dup-report.html")
        .unwrap();
    file.write_all(&html.into_bytes()).unwrap();
}
