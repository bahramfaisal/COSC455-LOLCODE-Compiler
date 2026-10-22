use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

pub trait LexicalAnalyzer {
    fn get_char(&mut self) -> char;
    fn add_char(&mut self, c: char);
    fn lookup(&self, s: &str) -> bool;
}

pub trait SyntaxAnalyzer {
    fn parse_lolcode(&mut self) -> Result<(), String>;
    fn parse_head(&mut self) -> Result<(), String>;
    fn parse_title(&mut self) -> Result<(), String>;
    fn parse_comment(&mut self) -> Result<(), String>;
    fn parse_body(&mut self) -> Result<(), String>;
    fn parse_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_text(&mut self) -> Result<(), String>;
    fn parse_variable_define(&mut self) -> Result<(), String>;
    fn parse_variable_use(&mut self) -> Result<(), String>;
    fn parse_bold(&mut self) -> Result<(), String>;
    fn parse_list(&mut self) -> Result<(), String>;
    fn parse_list_items(&mut self) -> Result<(), String>;
    fn parse_inner_list(&mut self) -> Result<(), String>;
    fn parse_link(&mut self) -> Result<(), String>;
    fn parse_newline(&mut self) -> Result<(), String>;
    fn parse_text(&mut self) -> Result<(), String>;
}

struct Lexer {
    chars: Vec<char>,
    position: usize,
    word: String,
}

impl LexicalAnalyzer for Lexer {
    fn get_char(&mut self) -> char {
        if self.position >= self.chars.len() {
            println!("Error: End of file");
            std::process::exit(1);
        }
        let c = self.chars[self.position];
        self.position += 1;
        c
    }

    fn add_char(&mut self, c: char) {
        self.word.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        let commands = [
            "#HAI", "#KBYE", "#OBTW", "#TLDR", "#MKAY", "#OIC",
            "#NEWLINE", "#IHAZ", "#ITIZ", "#MAEK HEAD",
            "#MAEK PARAGRAF", "#MAEK LIST", "#GIMMEH TITLE",
            "#GIMMEH BOLD", "#GIMMEH ITEM", "#GIMMEH LINX",
            "#GIMMEH VAR",
        ];
        commands.contains(&s)
    }
}

impl Lexer {
    fn next_token(&mut self) -> Option<String> {
        if self.position >= self.chars.len() {
            return None;
        }
        self.word.clear();

        if self.chars[self.position] == '#' {
            while self.position < self.chars.len() {
                let c = self.chars[self.position];
                if c.is_whitespace() || (c == '#' && !self.word.is_empty()) {
                    break;
                }
                let c = self.get_char();
                self.add_char(c);
            }
            let first = self.word.to_uppercase();

            if first == "#MAEK" || first == "#GIMMEH" {
                while self.position < self.chars.len()
                    && (self.chars[self.position] == ' '
                        || self.chars[self.position] == '\t')
                {
                    self.get_char();
                }
                let mut second = String::new();
                while self.position < self.chars.len() {
                    let c = self.chars[self.position];
                    if c.is_whitespace() || c == '#' {
                        break;
                    }
                    second.push(self.get_char());
                }
                let command = format!("{} {}", first, second.to_uppercase());
                if !self.lookup(&command) {
                    println!("Lexical error: {}", command);
                    std::process::exit(1);
                }
                return Some(command);
            }

            if !self.lookup(&first) {
                println!("Lexical error: {}", first);
                std::process::exit(1);
            }
            return Some(first);
        }

        while self.position < self.chars.len() {
            if self.chars[self.position] == '#' {
                break;
            }
            let c = self.get_char();
            self.add_char(c);
        }
        Some(self.word.clone())
    }
}

struct Parser {
    tokens: Vec<String>,
    position: usize,
    output: String,
    spaces_before: Vec<bool>,
    // Each block has its own variables. The last one is the current block.
    scopes: Vec<HashMap<String, String>>,
}

impl Parser {
    fn add_inline_space(&mut self) {
        let had_space = self.spaces_before.get(self.position).copied().unwrap_or(false);
        let ends_with_word = match self.output.chars().last() {
            Some(c) => c.is_alphanumeric() || ",.!?:%\"".contains(c),
            None => false,
        };
        let ends_with_inline_tag = self.output.ends_with("</b>")
            || self.output.ends_with("</a>");

        if had_space && (ends_with_word || ends_with_inline_tag) {
            self.output.push(' ');
        }
    }

    fn current_token(&self) -> Option<&str> {
        self.tokens.get(self.position).map(String::as_str)
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        if self.current_token() == Some(expected) {
            self.position += 1;
            Ok(())
        } else {
            let found = self.current_token().unwrap_or("end of file");
            Err(format!("Syntax error: Expected {}, found {}", expected, found))
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn find_variable(&self, name: &str) -> Option<String> {
        // Search the innermost block first, then outer blocks.
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    fn read_word(&mut self, kind: &str) -> Result<String, String> {
        let token = self.current_token().unwrap_or("").trim();
        if token.is_empty() || token.starts_with('#') || token.split_whitespace().count() != 1 {
            return Err(format!("Syntax error: {} must be one word", kind));
        }
        let word = token.to_string();
        self.position += 1;
        Ok(word)
    }

    fn parse_variable_define(&mut self) -> Result<(), String> {
        self.expect("#IHAZ")?;
        let name = self.read_word("Variable name")?;
        self.expect("#ITIZ")?;
        let value = self.current_token().unwrap_or("").trim();
        if value.is_empty() || value.starts_with('#') {
            return Err(String::from("Syntax error: Variable needs a value"));
        }
        let value = value.to_string();
        self.position += 1;
        self.expect("#MKAY")?;
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
        Ok(())
    }

    fn parse_variable_use(&mut self) -> Result<(), String> {
        self.add_inline_space();
        self.expect("#GIMMEH VAR")?;
        let name = self.read_word("Variable name")?;
        self.expect("#OIC")?;
        match self.find_variable(&name) {
            Some(value) => {
                self.output.push_str(&value);
                Ok(())
            }
            None => Err(format!("Semantic error: Undefined variable {}", name)),
        }
    }

    fn parse_lolcode(&mut self) -> Result<(), String> {
        self.expect("#HAI")?;
        self.output.push_str("<html>\n");
        self.enter_scope();

        while self.current_token() == Some("#OBTW") {
            self.parse_comment()?;
        }
        if self.current_token() == Some("#MAEK HEAD") {
            self.parse_head()?;
        }
        if self.current_token() == Some("#IHAZ") {
            self.parse_variable_define()?;
        }
        self.parse_body()?;
        self.expect("#KBYE")?;
        if self.position != self.tokens.len() {
            return Err(String::from("Syntax error: Extra text after #KBYE"));
        }
        self.leave_scope();
        self.output.push_str("\n</html>\n");
        Ok(())
    }

    fn parse_head(&mut self) -> Result<(), String> {
        self.expect("#MAEK HEAD")?;
        self.output.push_str("<head>\n");
        self.parse_title()?;
        self.expect("#MKAY")?;
        self.output.push_str("</head>\n");
        Ok(())
    }

    fn parse_title(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH TITLE")?;
        let title = self.current_token().unwrap_or("");
        if title.starts_with('#') || title.trim().is_empty() {
            return Err(String::from("Syntax error: Title needs text"));
        }
        self.output.push_str("<title>");
        self.parse_text();
        self.expect("#OIC")?;
        self.output.push_str("</title>\n");
        Ok(())
    }

    fn parse_comment(&mut self) -> Result<(), String> {
        self.expect("#OBTW")?;
        self.output.push_str("<!--");
        while self.position < self.tokens.len()
            && self.current_token() != Some("#TLDR")
        {
            if self.current_token() == Some("#KBYE") {
                return Err(String::from("Syntax error: Expected #TLDR, found #KBYE"));
            }
            if self.current_token().unwrap_or("").starts_with('#') {
                return Err(String::from("Syntax error: No commands inside comment"));
            }
            self.parse_text();
        }
        self.expect("#TLDR")?;
        self.output.push_str("-->\n");
        Ok(())
    }

    fn parse_body(&mut self) -> Result<(), String> {
        while self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            match token.as_str() {
                "#KBYE" => break,
                "#MAEK PARAGRAF" => self.parse_paragraph()?,
                "#MAEK LIST" => self.parse_list()?,
                "#GIMMEH BOLD" => self.parse_bold()?,
                "#GIMMEH LINX" => self.parse_link()?,
                "#GIMMEH VAR" => self.parse_variable_use()?,
                "#NEWLINE" => self.parse_newline()?,
                "#OBTW" => self.parse_comment()?,
                _ if !token.starts_with('#') => self.parse_text(),
                _ => return Err(format!("Syntax error: {} is not allowed here", token)),
            }
        }
        Ok(())
    }

    fn parse_paragraph(&mut self) -> Result<(), String> {
        self.expect("#MAEK PARAGRAF")?;
        self.enter_scope();
        self.output.push_str("<p>");
        if self.current_token() == Some("#IHAZ") {
            self.parse_variable_define()?;
        }
        SyntaxAnalyzer::parse_inner_paragraph(self)?;
        self.expect("#MKAY")?;
        self.leave_scope();
        self.output.push_str("</p>\n");
        Ok(())
    }

    fn parse_bold(&mut self) -> Result<(), String> {
        self.add_inline_space();
        self.expect("#GIMMEH BOLD")?;
        self.enter_scope();
        self.output.push_str("<b>");
        if self.current_token() == Some("#IHAZ") {
            self.parse_variable_define()?;
        }
        let mut found_text = false;
        while self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            match token.as_str() {
                "#OIC" => break,
                "#GIMMEH VAR" => {
                    self.parse_variable_use()?;
                    found_text = true;
                }
                _ if !token.starts_with('#') => {
                    if !token.trim().is_empty() {
                        found_text = true;
                    }
                    self.parse_text();
                }
                _ => return Err(format!("Syntax error: Expected #OIC, found {}", token)),
            }
        }
        if !found_text {
            return Err(String::from("Syntax error: Bold needs text"));
        }
        self.expect("#OIC")?;
        self.leave_scope();
        self.output.push_str("</b>");
        Ok(())
    }

    fn parse_list(&mut self) -> Result<(), String> {
        self.expect("#MAEK LIST")?;
        self.enter_scope();
        self.output.push_str("<ul>\n");
        if self.current_token() == Some("#IHAZ") {
            self.parse_variable_define()?;
        }
        SyntaxAnalyzer::parse_list_items(self)?;
        self.expect("#MKAY")?;
        self.leave_scope();
        self.output.push_str("</ul>\n");
        Ok(())
    }

    fn parse_list_item(&mut self) -> Result<(), String> {
        self.expect("#GIMMEH ITEM")?;
        self.enter_scope();
        self.output.push_str("<li>");
        if self.current_token() == Some("#IHAZ") {
            self.parse_variable_define()?;
        }
        SyntaxAnalyzer::parse_inner_list(self)?;
        self.expect("#OIC")?;
        self.leave_scope();
        self.output.push_str("</li>\n");
        Ok(())
    }

    fn parse_link(&mut self) -> Result<(), String> {
        self.add_inline_space();
        self.expect("#GIMMEH LINX")?;
        let address = self.current_token().unwrap_or("").trim().to_string();
        if address.is_empty() || address.contains(char::is_whitespace) || address.starts_with('#') {
            return Err(String::from("Syntax error: Link needs one web address"));
        }
        self.position += 1;
        self.expect("#OIC")?;
        self.output.push_str(&format!("<a href=\"{}\">{}</a>", address, address));
        Ok(())
    }

    fn parse_newline(&mut self) -> Result<(), String> {
        self.expect("#NEWLINE")?;
        self.output.push_str("<br>\n");
        Ok(())
    }

    fn parse_text(&mut self) {
        self.add_inline_space();
        let text = self.tokens[self.position]
            .trim_matches(|c| c == '\n' || c == '\r');
        self.output.push_str(text);
        self.position += 1;
    }
}

impl SyntaxAnalyzer for Parser {
    fn parse_lolcode(&mut self) -> Result<(), String> {
        Parser::parse_lolcode(self)
    }

    fn parse_head(&mut self) -> Result<(), String> {
        Parser::parse_head(self)
    }

    fn parse_title(&mut self) -> Result<(), String> {
        Parser::parse_title(self)
    }

    fn parse_comment(&mut self) -> Result<(), String> {
        Parser::parse_comment(self)
    }

    fn parse_body(&mut self) -> Result<(), String> {
        Parser::parse_body(self)
    }

    fn parse_paragraph(&mut self) -> Result<(), String> {
        Parser::parse_paragraph(self)
    }

    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        while self.position < self.tokens.len()
            && self.current_token() != Some("#MKAY")
        {
            SyntaxAnalyzer::parse_inner_text(self)?;
        }
        Ok(())
    }

    fn parse_inner_text(&mut self) -> Result<(), String> {
        let token = self.current_token().unwrap_or("end of file").to_string();
        match token.as_str() {
            "#GIMMEH VAR" => Parser::parse_variable_use(self),
            "#OBTW" => Parser::parse_comment(self),
            "#GIMMEH BOLD" => Parser::parse_bold(self),
            "#MAEK LIST" => Parser::parse_list(self),
            "#GIMMEH LINX" => Parser::parse_link(self),
            "#NEWLINE" => Parser::parse_newline(self),
            _ if !token.starts_with('#') && self.current_token().is_some() => {
                Parser::parse_text(self);
                Ok(())
            }
            _ => Err(format!(
                "Syntax error: {} is not allowed inside paragraph", token
            )),
        }
    }

    fn parse_variable_define(&mut self) -> Result<(), String> {
        Parser::parse_variable_define(self)
    }

    fn parse_variable_use(&mut self) -> Result<(), String> {
        Parser::parse_variable_use(self)
    }

    fn parse_bold(&mut self) -> Result<(), String> {
        Parser::parse_bold(self)
    }

    fn parse_list(&mut self) -> Result<(), String> {
        Parser::parse_list(self)
    }

    fn parse_list_items(&mut self) -> Result<(), String> {
        let mut count = 0;
        while self.current_token() == Some("#GIMMEH ITEM") {
            self.parse_list_item()?;
            count += 1;
        }
        if count == 0 {
            return Err(String::from("Syntax error: List needs an item"));
        }
        Ok(())
    }

    fn parse_inner_list(&mut self) -> Result<(), String> {
        while self.position < self.tokens.len()
            && self.current_token() != Some("#OIC")
        {
            let token = self.current_token().unwrap_or("end of file").to_string();
            match token.as_str() {
                "#GIMMEH BOLD" => Parser::parse_bold(self)?,
                "#GIMMEH LINX" => Parser::parse_link(self)?,
                "#GIMMEH VAR" => Parser::parse_variable_use(self)?,
                "#NEWLINE" => Parser::parse_newline(self)?,
                "#OBTW" => Parser::parse_comment(self)?,
                _ if !token.starts_with('#') => Parser::parse_text(self),
                _ => return Err(format!(
                    "Syntax error: Expected #OIC, found {}", token
                )),
            }
        }
        Ok(())
    }

    fn parse_link(&mut self) -> Result<(), String> {
        Parser::parse_link(self)
    }

    fn parse_newline(&mut self) -> Result<(), String> {
        Parser::parse_newline(self)
    }

    fn parse_text(&mut self) -> Result<(), String> {
        if self.current_token().is_none() {
            return Err(String::from("Syntax error: Expected text, found end of file"));
        }
        Parser::parse_text(self);
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 || !args[1].ends_with(".lol") {
        println!("Usage: lolcompiler input.lol");
        return;
    }

    let file_name = &args[1];
    let text = match fs::read_to_string(file_name) {
        Ok(text) => text,
        Err(_) => {
            println!("Error: Cannot read file");
            return;
        }
    };

    let output_path = Path::new(file_name).with_extension("html");
    let _ = fs::remove_file(&output_path);

    let mut lexer = Lexer {
        chars: text.chars().collect(),
        position: 0,
        word: String::new(),
    };
    let mut tokens = Vec::new();
    let mut spaces_before = Vec::new();
    let mut skipped_space = false;

    while let Some(token) = lexer.next_token() {
        if token.trim().is_empty() {
            skipped_space = true;
        } else {
            let starts_with_space = token.chars().next()
                .map_or(false, |c| c.is_whitespace());
            spaces_before.push(skipped_space || starts_with_space);
            skipped_space = token.chars().last()
                .map_or(false, |c| c.is_whitespace());
            tokens.push(token);
        }
    }

    let mut parser = Parser {
        tokens,
        position: 0,
        output: String::new(),
        spaces_before,
        scopes: Vec::new(),
    };

    match SyntaxAnalyzer::parse_lolcode(&mut parser) {
        Ok(()) => match fs::write(&output_path, &parser.output) {
            Ok(()) => {
                println!("Basic parser passed!");
                println!("HTML output:\n{}", parser.output);
                println!("Saved HTML to {}", output_path.display());
                if cfg!(target_os = "macos") {
                    let _ = Command::new("open").arg(&output_path).status();
                }
            }
            Err(_) => println!("Error: Could not save HTML"),
        },
        Err(error) => println!("{}", error),
    }
}
