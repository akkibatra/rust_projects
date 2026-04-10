use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run -- <file.md>");
        return;
    }

    let input_path = &args[1];

    let contents = fs::read_to_string(input_path).unwrap_or_else(|err| {
        eprintln!("Error reading file {}: {}", input_path, err);
        process::exit(1);
    });

    let html_output = markdown_to_html(&contents);
    fs::write("output.html", html_output).expect("Unable to write to file");
    println!("Successfully converted {} to output.html", input_path);
}

fn markdown_to_html(markdown: &str) -> String {
    // Simple markdown to HTML conversion (replace with actual implementation)
    let mut html = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") {
            let content = &trimmed[2..];
            html.push_str(&format!("<h1>{}</h1>\n", content));
        } else if trimmed.starts_with("## ") {
            let content = &trimmed[2..];
            html.push_str(&format!("<h2>{}</h2>\n", content));
        } else if trimmed.starts_with("### ") {
            let content = &trimmed[2..];
            html.push_str(&format!("<h3>{}</h3>\n", content));
        } else if trimmed.is_empty(){
            continue;
        } else {
            let content = trimmed.replace("**", "<b>").replace("**", "</b>");
            html.push_str(&format!("<p>{}</p>\n", content));
        }
    }

    format!("<!DOCTYPE html>\n<html>\n<head>\n    <title>Document</title>\n</head>\n<body>\n{}\n</body>\n</html>", html)
}
