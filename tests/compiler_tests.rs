
use std::fs;
use std::process::Command;

fn run_test(name: &str, code: &str) -> (String, Option<String>) {
    let folder = std::env::temp_dir();
    let file = folder.join(format!(
        "lol_test_{}_{}.lol",
        std::process::id(),
        name
    ));

    fs::write(&file, code).unwrap();

    let html_file = file.with_extension("html");
    let _ = fs::remove_file(&html_file);

    let result = Command::new(env!("CARGO_BIN_EXE_lolcompiler"))
        .arg(&file)
        .output()
        .unwrap();

    let message = String::from_utf8_lossy(&result.stdout).to_string();
    let html = fs::read_to_string(&html_file).ok();

    (message, html)
}

#[test]
fn test_valid_html() {
    let code = "#HAI
#IHAZ lastname #ITIZ Simpson #MKAY
#MAEK PARAGRAF
Hello #GIMMEH BOLD #GIMMEH VAR lastname #OIC #OIC
#MKAY
#KBYE";

    let (message, html) = run_test("valid", code);

    assert!(message.contains("Basic parser passed!"));
    assert!(html.unwrap().contains("<p>Hello <b>Simpson</b></p>"));
}

#[test]
fn test_missing_hai() {
    let code = "Hello world
#KBYE";

    let (message, html) = run_test("missing_hai", code);

    assert!(message.contains("Expected #HAI"));
    assert!(html.is_none());
}

#[test]
fn test_undefined_variable() {
    let code = "#HAI
Hello #GIMMEH VAR myname #OIC
#KBYE";

    let (message, html) = run_test("undefined", code);

    assert!(message.contains("Undefined variable myname"));
    assert!(html.is_none());
}

#[test]
fn test_missing_comment_end() {
    let code = "#HAI
#OBTW
This is my comment
#KBYE";

    let (message, html) = run_test("comment", code);

    assert!(message.contains("Expected #TLDR"));
    assert!(html.is_none());
}
