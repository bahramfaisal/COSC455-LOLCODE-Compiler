
# COSC 455 - Project 1
## LOLCODE Markdown Compiler

### Project Description
This project is a LOLCODE compiler written in Rust. It reads a .lol file, checks the commands, and converts the code into an HTML file.

### Operating System
macOS

### Programming Language
Rust

### How to Run
1. Open the project folder in Terminal.
2. Run this command:

   cargo run -- src/input.lol

3. The compiler checks the LOLCODE program.
4. If the code is correct, it creates input.html.
5. The HTML file opens in a web browser on macOS.

### How to Test
Run this command:

   cargo test

The project has automatic tests for valid HTML, missing commands, and undefined variables.

### Features
- Lexical analysis
- Syntax analysis
- HTML generation
- Paragraphs and bold text
- Lists and links
- Comments
- Variables and variable scope
- Syntax and semantic error messages

### Project Files
- src/main.rs - Compiler source code
- src/input.lol - Example input
- tests/compiler_tests.rs - Automatic tests
- Cargo.toml - Rust project settings

### GitHub Repository
https://github.com/bahramfaisal/COSC455-LOLCODE-Compiler
