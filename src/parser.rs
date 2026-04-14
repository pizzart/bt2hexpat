use crate::ast::*;
use std::collections::VecDeque;

pub struct Parser {
    tokens: VecDeque<String>,
    pos: usize,
    lines: Vec<String>,
    context_stack: Vec<String>, // Track nested contexts
}

impl Parser {
    pub fn new(content: &str) -> Self {
        let tokens = Self::tokenize(content);
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        Parser {
            tokens,
            pos: 0,
            lines,
            context_stack: Vec::new(),
        }
    }

    fn push_context(&mut self, context: &str) {
        self.context_stack.push(context.to_string());
        eprintln!("[DEBUG] Entering {}", context);
    }

    fn pop_context(&mut self) {
        if let Some(context) = self.context_stack.pop() {
            eprintln!("[DEBUG] Exiting {}", context);
        }
    }

    fn get_context_stack(&self) -> String {
        if self.context_stack.is_empty() {
            "top-level".to_string()
        } else {
            self.context_stack.join(" > ")
        }
    }

    fn get_context(&self) -> String {
        let line_num = self.get_line_number();
        let context = if line_num > 0 && line_num <= self.lines.len() {
            let line = &self.lines[line_num - 1];
            let trimmed = line.trim();
            format!(
                "  Line {}: {}\n  Block context: {}",
                line_num,
                trimmed,
                self.get_context_stack()
            )
        } else {
            format!("  Block context: {}", self.get_context_stack())
        };
        context
    }

    fn get_line_number(&self) -> usize {
        let mut line = 1;
        for i in 0..self.pos.min(self.tokens.len()) {
            if self.tokens[i] == "\n" {
                line += 1;
            }
        }
        line
    }

    fn tokenize(content: &str) -> VecDeque<String> {
        let mut tokens = VecDeque::new();
        let mut current = String::new();
        let mut in_comment = false;
        let mut in_line_comment = false;
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if in_line_comment {
                if ch == '\n' {
                    in_line_comment = false;
                    // tokens.push_back("\n".to_string());
                }
                continue;
            }

            if in_comment {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_comment = false;
                }
                continue;
            }

            match ch {
                '/' if chars.peek() == Some(&'/') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    in_line_comment = true;
                }
                '/' if chars.peek() == Some(&'*') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    in_comment = true;
                }
                '!' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("!=".to_string());
                }
                '=' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("==".to_string());
                }
                '<' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("<=".to_string());
                }
                '>' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back(">=".to_string());
                }
                '&' if chars.peek() == Some(&'&') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("&&".to_string());
                }
                '|' if chars.peek() == Some(&'|') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("||".to_string());
                }
                '+' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("+=".to_string());
                }
                '-' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("-=".to_string());
                }
                '*' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("*=".to_string());
                }
                '/' if chars.peek() == Some(&'=') => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    chars.next();
                    tokens.push_back("/=".to_string());
                }
                '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '=' | '<' | '>' | '+' | '-'
                | '*' | '/' | '&' | '|' | '!' => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    tokens.push_back(ch.to_string());
                }
                '"' => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    let mut string_content = String::from("\"");
                    while let Some(str_ch) = chars.next() {
                        string_content.push(str_ch);
                        if str_ch == '"' {
                            break;
                        }
                    }
                    tokens.push_back(string_content);
                }
                '\'' => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    let mut char_content = String::from("'");
                    while let Some(ch_char) = chars.next() {
                        char_content.push(ch_char);
                        if ch_char == '\'' {
                            break;
                        }
                    }
                    tokens.push_back(char_content);
                }
                '\n' => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                    // tokens.push_back("\n".to_string());
                }
                ' ' | '\t' | '\r' => {
                    if !current.is_empty() {
                        tokens.push_back(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push_back(current);
        }

        tokens
    }

    pub fn parse(&mut self) -> Result<Template, String> {
        let mut statements = Vec::new();
        self.push_context("top-level");

        while !self.is_eof() {
            let token = self.peek_token().to_lowercase();

            match token.as_str() {
                "typedef" => {
                    eprintln!("[DEBUG] Reading typedef");
                    self.push_context("typedef");
                    statements.push(self.parse_typedef()?);
                    self.pop_context();
                }
                "struct" => {
                    eprintln!("[DEBUG] Reading struct");
                    self.push_context("struct");
                    statements.push(self.parse_struct()?);
                    self.pop_context();
                }
                "enum" => {
                    eprintln!("[DEBUG] Reading enum");
                    self.push_context("enum");
                    statements.push(self.parse_enum()?);
                    self.pop_context();
                }
                "local" => {
                    eprintln!("[DEBUG] Reading local variable");
                    self.push_context("local variable");
                    statements.push(self.parse_var_decl(true)?);
                    self.pop_context();
                }
                "if" => {
                    eprintln!("[DEBUG] Reading if statement");
                    statements.push(self.parse_if()?);
                }
                "while" => {
                    eprintln!("[DEBUG] Reading while loop");
                    statements.push(self.parse_while()?);
                }
                "warning" | "printf" | "littleendian" | "bigendian" | "requiresversion" => {
                    eprintln!("[DEBUG] Reading function call: {}", token);
                    self.parse_function_call()?;
                }
                _ if self.is_type_start(&token) => {
                    eprintln!("[DEBUG] Reading variable declaration with type: {}", token);
                    statements.push(self.parse_var_decl(false)?);
                }
                _ => {
                    eprintln!("[DEBUG] Reading expression statement: {}", token);
                    self.push_context("expression statement");
                    match self.parse_expression_statement()? {
                        Some(s) => statements.push(s),
                        None => (),
                    }
                    self.pop_context();
                }
            }
        }

        self.pop_context();
        Ok(Template {
            statements,
            metadata: TemplateMetadata::default(),
        })
    }

    fn parse_expression_statement(&mut self) -> Result<Option<Statement>, String> {
        // let start_pos = self.pos;

        let expr = self.parse_primary_expr()?;
        dbg!(&expr);
        match self.peek_token() {
            "=" => {
                self.advance();
                let right = self.parse_expr()?;
                self.expect(";")?;
                Ok(Some(Statement::Assign { left: expr, right }))
            }
            _ => match expr {
                Expression::FunctionCall(ident, args) => {
                    self.advance();
                    Ok(Some(Statement::FunctionCall(ident, args)))
                }
                _ => {
                    while self.peek_token() != ";" && !self.is_eof() {
                        self.advance();
                    }
                    eprintln!("[DEBUG] Meaningless statement {}", self.get_context());
                    Ok(None)
                }
            },
        }

        // let mut depth = 0;
        // while !self.is_eof() {
        //     let token = self.peek_token();

        //     match token {
        //         // "(" | "{" | "[" => depth += 1,
        //         // ")" | "}" | "]" => depth -= 1,
        //         ";" if depth == 0 => {
        //             self.advance();
        //             eprintln!(
        //                 "[DEBUG] Finished expression statement at position {}",
        //                 self.pos
        //             );
        //             return Ok(());
        //         }
        //         _ => {}
        //     }

        //     self.advance();
        // }

        // Ok(())
    }

    fn parse_typedef(&mut self) -> Result<Statement, String> {
        self.expect("typedef")?;

        if self.peek_token().to_lowercase() == "struct" {
            eprintln!("[DEBUG] Parsing typedef struct");
            self.expect("struct")?;

            let struct_name = if self.peek_token() != "{" {
                let name = self.peek_token().to_string();
                self.advance();

                name
            } else {
                "".to_string()
            };

            self.expect("{")?;

            self.push_context(&format!("typedef struct {} fields", struct_name));
            let fields = self.parse_struct_contents()?;
            self.pop_context();

            self.expect("}")?;

            if self.peek_token() == "<" {
                self.skip_until(">")?;
            }

            let type_name = self.peek_token().to_string();
            eprintln!("[DEBUG] Typedef struct name: {}", type_name);
            self.advance();

            if self.peek_token() == "<" {
                self.skip_until(">")?;
            }

            self.expect(";")?;

            Ok(Statement::StructDef {
                ident: type_name,
                fields,
            })
        } else {
            eprintln!("[DEBUG] Parsing typedef type alias");
            let base_ty = self
                .parse_type()
                .map_err(|e| format!("{}. Context: parsing typedef{}", e, self.get_context()))?;

            let name = self.peek_token().to_string();
            eprintln!("[DEBUG] Typedef name: {}", name);
            self.advance();

            let mut ty = base_ty;
            while self.peek_token() == "[" {
                self.advance();

                let size = if self.peek_token() != "]" {
                    Some(Box::new(self.parse_expr().map_err(|e| {
                        format!(
                            "{}. Context: parsing typedef array dimension{}",
                            e,
                            self.get_context()
                        )
                    })?))
                } else {
                    None
                };

                self.expect("]").map_err(|e| {
                    format!(
                        "{}. Context: in typedef array definition{}",
                        e,
                        self.get_context()
                    )
                })?;

                ty = DataType::Array(Box::new(ty), size);
            }

            self.expect(";")
                .map_err(|e| format!("{}. Context: at end of typedef statement. Current token: '{}' at position {}. Expected ';'", e, self.peek_token(), self.pos))?;

            Ok(Statement::TypeDef { ident: name, ty })
        }
    }

    fn parse_struct(&mut self) -> Result<Statement, String> {
        self.expect("struct")?;

        let name = self.peek_token().to_string();
        eprintln!("[DEBUG] Parsing struct: {}", name);
        self.advance();

        if self.peek_token() == "<" {
            self.skip_until(">")?;
        }

        self.expect("{")?;
        eprintln!("[DEBUG] Entered struct body");

        self.push_context(&format!("struct {} fields", name));
        let fields = self.parse_struct_contents()?;
        self.pop_context();

        eprintln!(
            "[DEBUG] About to close struct, current token: '{}'",
            self.peek_token()
        );

        self.expect("}")?;
        eprintln!("[DEBUG] Struct closed successfully");

        // Handle optional style annotation
        if self.peek_token() == "<" {
            self.skip_until(">")?;
        }

        // Optional type name after closing brace
        if self.peek_token() != ";"
            && !self.is_eof()
            && self.peek_token() != "local"
            && !self.is_type_start(self.peek_token())
        {
            self.advance();
        }

        if self.peek_token() == ";" {
            self.advance();
        }

        Ok(Statement::StructDef {
            ident: name,
            fields,
        })
    }

    fn parse_struct_contents(&mut self) -> Result<Vec<StructContent>, String> {
        let mut contents = Vec::new();

        while self.peek_token() != "}" && !self.is_eof() {
            // if self.peek() == "}" || self.is_eof() {
            //     break;
            // }

            let token_lower = self.peek_token().to_lowercase();

            // Handle local statements inside structs - skip them
            if token_lower == "local" {
                eprintln!("[DEBUG] Parsing local variable in struct");
                self.push_context("local");
                self.advance();

                while self.peek_token() != ";" && !self.is_eof() {
                    self.advance();
                }

                if self.peek_token() == ";" {
                    self.advance();
                }

                self.pop_context();
                continue;
            }

            // Handle if statements inside structs
            if token_lower == "if" {
                eprintln!("[DEBUG] Parsing if statement in struct");
                self.push_context("if statement");
                let if_stmt = self.parse_if()?;
                self.pop_context();

                // After parsing if, check for else/else if and consume them
                while self.peek_token().to_lowercase() == "else" {
                    eprintln!("[DEBUG] Parsing else clause after if in struct");
                    self.advance();

                    // Check for "else if"
                    if self.peek_token().to_lowercase() == "if" {
                        eprintln!("[DEBUG] Parsing else if in struct");
                        self.advance();

                        self.expect("(")?;

                        self.parse_expr()?;

                        self.expect(")")?;
                    }

                    // Parse the block or statement
                    if self.peek_token() == "{" {
                        self.advance();

                        let mut brace_depth = 1;
                        while brace_depth > 0 && !self.is_eof() {
                            match self.peek_token() {
                                "{" => brace_depth += 1,
                                "}" => brace_depth -= 1,
                                _ => {}
                            }
                            if brace_depth > 0 {
                                self.advance();
                            }
                        }
                        self.advance(); // consume final }
                    } else {
                        // Single statement
                        while self.peek_token() != ";" && !self.is_eof() {
                            self.advance();
                        }
                        if self.peek_token() == ";" {
                            self.advance();
                        }
                    }
                }

                eprintln!("[DEBUG] Adding if statement to struct");
                contents.push(StructContent::Statement(Box::new(if_stmt)));
                continue;
            }

            // Handle while loops inside structs
            if token_lower == "while" {
                eprintln!("[DEBUG] Parsing while loop in struct");
                self.push_context("while loop");
                let while_stmt = self.parse_while()?;
                self.pop_context();

                eprintln!("[DEBUG] Adding while statement to struct");
                contents.push(StructContent::Statement(Box::new(while_stmt)));
                continue;
            }

            // Handle switch statements inside structs
            if token_lower == "switch" {
                eprintln!("[DEBUG] Parsing switch statement in struct");
                self.push_context("switch statement");
                let switch_stmt = self.parse_switch()?;
                self.pop_context();

                eprintln!("[DEBUG] Adding switch statement to struct");
                contents.push(StructContent::Statement(Box::new(switch_stmt)));
                continue;
            }

            // Handle expression statements inside structs (like reassignments)
            if self.is_expression_statement_start() {
                eprintln!("[DEBUG] Parsing expression statement in struct");
                self.push_context("expression statement");
                match self.parse_expression_statement()? {
                    Some(s) => contents.push(StructContent::Statement(Box::new(s))),
                    None => (),
                }
                self.pop_context();

                continue;
            }

            eprintln!("[DEBUG] Parsing struct field");
            let field = self.parse_struct_field()?;
            contents.push(StructContent::Field(field));
        }

        Ok(contents)
    }

    fn parse_struct_field(&mut self) -> Result<StructField, String> {
        let base_ty = self.parse_type().map_err(|e| {
            format!(
                "{}. Context: parsing struct field type{}",
                e,
                self.get_context()
            )
        })?;

        let name = self.peek_token().to_string();
        eprintln!("[DEBUG] Struct field name: {}", name);
        self.advance();

        let mut ty = base_ty;
        while self.peek_token() == "[" {
            self.advance();

            let size = if self.peek_token() != "]" {
                Some(Box::new(self.parse_expr().map_err(|e| {
                    format!(
                        "{}. Context: parsing array size for field '{}'{}",
                        e,
                        name,
                        self.get_context()
                    )
                })?))
            } else {
                None
            };

            self.expect("]").map_err(|e| {
                format!(
                    "{}. Context: in array definition for field '{}'{}",
                    e,
                    name,
                    self.get_context()
                )
            })?;

            ty = DataType::Array(Box::new(ty), size);
        }

        let mut condition = None;

        if self.peek_token().to_lowercase() == "if" {
            self.advance();

            condition = Some(self.parse_condition().map_err(|e| {
                format!(
                    "{}. Context: parsing field condition{}",
                    e,
                    self.get_context()
                )
            })?);
        }

        if self.peek_token() == "<" {
            self.skip_until(">")?;
        }

        if self.peek_token() == ";" {
            self.advance();
        } else if self.peek_token() != "}"
            && !self.is_eof()
            && self.peek_token().to_lowercase() != "local"
            && self.peek_token().to_lowercase() != "if"
            && self.peek_token().to_lowercase() != "else"
            && self.peek_token().to_lowercase() != "while"
            && self.peek_token().to_lowercase() != "switch"
        {
            return Err(format!(
                "Expected ';' but found '{}' at token position {} (line {}). Context: struct field type={}, name={}{}",
                self.peek_token(),
                self.pos,
                self.get_line_number(),
                match &ty {
                    DataType::Custom(c) => c.clone(),
                    _ => format!("{:?}", ty),
                },
                name,
                self.get_context()
            ));
        }

        return Ok(StructField {
            ident: name,
            ty,
            condition,
            comment: None,
        });
    }

    fn is_expression_statement_start(&self) -> bool {
        // Look ahead to see if this is an expression statement
        // Expression statements typically start with an identifier followed by = or (
        let token = self.peek_token();

        // Check if it's a control flow keyword
        let lower = token.to_lowercase();
        if matches!(
            lower.as_str(),
            "if" | "else"
                | "while"
                | "for"
                | "switch"
                | "case"
                | "default"
                | "return"
                | "break"
                | "continue"
        ) {
            eprintln!(
                "[DEBUG] is_expression_statement_start: token '{}' is a control flow keyword, returning false",
                lower
            );
            return false;
        }

        // Check if it's a type keyword
        if self.is_type_start(token) {
            eprintln!(
                "[DEBUG] is_expression_statement_start: token '{}' is a type keyword, returning false",
                lower
            );
            return false;
        }

        // Check if it starts with a valid identifier character
        if token
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            // Look ahead for = or (
            if self.pos + 1 < self.tokens.len() {
                let next_pos = self.pos + 1;
                // Skip whitespace to find next meaningful token
                let mut check_pos = next_pos;
                while check_pos < self.tokens.len()
                    && (self.tokens[check_pos] == " "
                        || self.tokens[check_pos] == "\t"
                        || self.tokens[check_pos] == "\n")
                {
                    check_pos += 1;
                }

                if check_pos < self.tokens.len() {
                    let next_token = &self.tokens[check_pos];
                    let result = next_token == "=" || next_token == "(";
                    eprintln!(
                        "[DEBUG] is_expression_statement_start: token '{}' followed by '{}', returning {}",
                        token, next_token, result
                    );
                    return result;
                }
            }
        }

        eprintln!(
            "[DEBUG] is_expression_statement_start: token '{}' is not an expression statement start",
            token
        );
        false
    }

    fn parse_condition(&mut self) -> Result<Expression, String> {
        self.expect("(")?;

        let expr = self.parse_expr()?;

        self.expect(")")?;
        Ok(expr)
    }

    fn parse_enum(&mut self) -> Result<Statement, String> {
        self.expect("enum")?;

        let name = self.peek_token().to_string();
        eprintln!("[DEBUG] Parsing enum: {}", name);
        self.advance();

        if self.peek_token() == ":" {
            self.advance();

            self.parse_type()?;
        }

        self.expect("{")?;

        let mut variants = Vec::new();

        while self.peek_token() != "}" && !self.is_eof() {
            if self.peek_token() == "}" {
                break;
            }

            let variant_name = self.peek_token().to_string();
            eprintln!("[DEBUG] Enum variant: {}", variant_name);
            self.advance();

            let value = if self.peek_token() == "=" {
                self.advance();

                Some(self.parse_literal()?.parse::<i64>().unwrap_or(0))
            } else {
                None
            };

            variants.push((variant_name, value));

            if self.peek_token() == "," {
                self.advance();
            }
        }

        self.expect("}")?;

        self.expect(";")?;

        Ok(Statement::EnumDef {
            ident: name,
            variants,
        })
    }

    fn parse_var_decl(&mut self, local: bool) -> Result<Statement, String> {
        if local {
            self.expect("local")?;
        }

        let base_ty = self.parse_type().map_err(|e| {
            format!(
                "{}. Context: parsing variable type{}",
                e,
                self.get_context()
            )
        })?;

        let name = self.peek_token().to_string();
        eprintln!("[DEBUG] Variable name: {}", name);
        self.advance();

        let mut ty = base_ty;
        while self.peek_token() == "[" {
            self.advance();

            let size = if self.peek_token() != "]" {
                Some(Box::new(self.parse_expr().map_err(|e| {
                    format!(
                        "{}. Context: parsing array size for variable '{}'{}",
                        e,
                        name,
                        self.get_context()
                    )
                })?))
            } else {
                None
            };

            self.expect("]")?;

            ty = DataType::Array(Box::new(ty), size);
        }

        let value = if self.peek_token() == "=" {
            self.advance();

            Some(self.parse_expr().map_err(|e| {
                format!(
                    "{}. Context: parsing initialization value for variable '{}'{}",
                    e,
                    name,
                    self.get_context()
                )
            })?)
        } else {
            None
        };

        if self.peek_token() == ";" {
            self.advance();
        }

        Ok(Statement::VarDecl {
            ident: name,
            ty,
            value,
            local,
        })
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect("if")?;
        self.expect("(")?;

        eprintln!("[DEBUG] Parsing if condition");
        let condition = self
            .parse_expr()
            .map_err(|e| format!("{}. Context: parsing if condition{}", e, self.get_context()))?;

        self.expect(")")?;

        let then_block = if self.peek_token() == "{" {
            eprintln!("[DEBUG] Entering if block");
            self.advance();

            self.push_context("if block");
            let block = self.parse_block()?;
            self.pop_context();

            self.expect("}")?;
            block
        } else {
            eprintln!("[DEBUG] Entering one-line if");
            // self.advance();

            vec![self.parse_statement()?]
        };

        let else_block = if self.peek_token().to_lowercase() == "else" {
            self.advance();

            if self.peek_token() == "{" {
                eprintln!("[DEBUG] Entering else block");
                self.advance();

                self.push_context("else block");
                let block = self.parse_block()?;
                self.pop_context();

                self.expect("}")?;
                Some(block)
            } else {
                Some(vec![self.parse_statement()?])
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        self.expect("while")?;

        self.expect("(")?;

        eprintln!("[DEBUG] Parsing while condition");
        let condition = self.parse_expr().map_err(|e| {
            format!(
                "{}. Context: parsing while condition{}",
                e,
                self.get_context()
            )
        })?;

        self.expect(")")?;

        let body = if self.peek_token() == "{" {
            eprintln!("[DEBUG] Entering while block");
            self.advance();

            self.push_context("while block");
            let block = self.parse_block()?;
            self.pop_context();

            eprintln!(
                "[DEBUG] About to expect closing brace, current token: '{}'",
                self.peek_token()
            );
            self.expect("}")?;
            eprintln!("[DEBUG] Exiting while block");
            block
        } else {
            vec![self.parse_statement()?]
        };

        Ok(Statement::While { condition, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();

        while self.peek_token() != "}" && !self.is_eof() {
            if self.peek_token() == "}" || self.is_eof() {
                eprintln!(
                    "[DEBUG] parse_block: stopping, current token: '{}', eof: {}",
                    self.peek_token(),
                    self.is_eof()
                );
                break;
            }

            statements.push(self.parse_statement()?);
        }

        eprintln!(
            "[DEBUG] parse_block finished with {} statements, current token: '{}'",
            statements.len(),
            self.peek_token()
        );
        Ok(statements)
    }

    fn parse_switch(&mut self) -> Result<Statement, String> {
        self.expect("switch")?;

        self.expect("(")?;

        eprintln!("[DEBUG] Parsing switch expression");
        let expr = self.parse_expr()?;

        self.expect(")")?;

        self.expect("{")?;

        eprintln!("[DEBUG] Entering switch block");
        let mut cases = Vec::new();

        while self.peek_token() != "}" && !self.is_eof() {
            if self.peek_token() == "}" {
                break;
            }

            let token_lower = self.peek_token().to_lowercase();

            if token_lower == "case" || token_lower == "default" {
                if token_lower == "case" {
                    eprintln!("[DEBUG] Parsing case label");
                    self.advance();

                    // Skip the case value
                    while self.peek_token() != ":" && !self.is_eof() {
                        self.advance();
                    }
                } else {
                    eprintln!("[DEBUG] Parsing default label");
                    self.advance();
                }

                if self.peek_token() == ":" {
                    self.advance();
                }

                // Parse statements in case
                let mut case_body = Vec::new();
                while self.peek_token() != "case"
                    && self.peek_token() != "default"
                    && self.peek_token() != "}"
                    && !self.is_eof()
                {
                    if self.peek_token() == "case"
                        || self.peek_token() == "default"
                        || self.peek_token() == "}"
                    {
                        break;
                    }
                    case_body.push(self.parse_statement()?);
                }
                cases.push(case_body);
            } else {
                break;
            }
        }

        self.expect("}")?;
        eprintln!("[DEBUG] Exiting switch block");

        Ok(Statement::Switch { expr, cases })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek_token().to_lowercase();
        eprintln!("[DEBUG] parse_statement: token = '{}'", token);

        match token.as_str() {
            "if" => {
                self.push_context("if statement");
                let result = self.parse_if();
                self.pop_context();
                result
            }
            "while" => {
                self.push_context("while loop");
                let result = self.parse_while();
                self.pop_context();
                result
            }
            "switch" => {
                self.push_context("switch statement");
                eprintln!("[DEBUG] parse_statement: parsing switch statement");
                let result = self.parse_switch();
                self.pop_context();
                result
            }
            "return" => {
                eprintln!("[DEBUG] Parsing return statement");
                self.advance();

                let value = if self.peek_token() != ";" {
                    Some(self.parse_expr().map_err(|e| {
                        format!("{}. Context: parsing return value{}", e, self.get_context())
                    })?)
                } else {
                    None
                };
                dbg!(&value);

                if self.peek_token() == ";" {
                    self.advance();
                }
                Ok(Statement::Return(value))
            }
            "local" => {
                self.push_context("local variable");
                let result = self.parse_var_decl(true);
                self.pop_context();
                result
            }
            _ if self.is_type_start(&token) => {
                eprintln!("[DEBUG] parse_statement: parsing var decl");
                self.parse_var_decl(false)
            }
            _ => {
                eprintln!(
                    "[DEBUG] parse_statement: parsing expression statement, token: '{}'",
                    token
                );
                match self.parse_expression_statement()? {
                    Some(s) => Ok(s),
                    None => Ok(Statement::FunctionCall(String::new(), vec![])),
                }
            }
        }
    }

    fn parse_type(&mut self) -> Result<DataType, String> {
        let token = self.peek_token().to_lowercase();

        if token == "unsigned" {
            self.advance();

            let next_token = self.peek_token().to_lowercase();
            let base_type = match next_token.as_str() {
                "char" => {
                    self.advance();
                    DataType::UChar
                }
                "short" => {
                    self.advance();
                    DataType::UShort
                }
                "int" => {
                    self.advance();
                    DataType::UInt
                }
                "long" => {
                    self.advance();
                    if self.peek_token().to_lowercase() == "long" {
                        self.advance();
                        DataType::UQuad
                    } else {
                        DataType::ULong
                    }
                }
                _ => DataType::UInt,
            };
            return Ok(base_type);
        }

        if token == "signed" {
            self.advance();

            let next_token = self.peek_token().to_lowercase();
            let base_type = match next_token.as_str() {
                "char" => {
                    self.advance();
                    DataType::Char
                }
                "short" => {
                    self.advance();
                    DataType::Short
                }
                "int" => {
                    self.advance();
                    DataType::Int
                }
                "long" => {
                    self.advance();
                    if self.peek_token().to_lowercase() == "long" {
                        self.advance();
                        DataType::Quad
                    } else {
                        DataType::Long
                    }
                }
                _ => DataType::Int,
            };
            return Ok(base_type);
        }

        let base_type = match token.as_str() {
            "char" => DataType::Char,
            "uchar" => DataType::UChar,
            "short" => DataType::Short,
            "ushort" => DataType::UShort,
            "int" => DataType::Int,
            "uint" => DataType::UInt,
            "long" => DataType::Long,
            "ulong" => DataType::ULong,
            "quad" => DataType::Quad,
            "uquad" => DataType::UQuad,
            "float" => DataType::Float,
            "double" => DataType::Double,
            _ => DataType::Custom(token.to_string()),
        };

        self.advance();

        Ok(base_type)
    }

    fn parse_expr(&mut self) -> Result<Expression, String> {
        self.parse_ternary_expr()
    }

    fn parse_ternary_expr(&mut self) -> Result<Expression, String> {
        let expr = self.parse_binary_expr(0)?;

        if self.peek_token() == "?" {
            self.advance();

            let true_expr = self.parse_expr()?;

            self.expect(":")?;

            let false_expr = self.parse_expr()?;
            return Ok(Expression::BinaryOp(
                Box::new(expr),
                "?:".to_string(),
                Box::new(Expression::BinaryOp(
                    Box::new(true_expr),
                    ":".to_string(),
                    Box::new(false_expr),
                )),
            ));
        }

        Ok(expr)
    }

    fn parse_binary_expr(&mut self, min_prec: i32) -> Result<Expression, String> {
        let mut left = self.parse_primary_expr()?;

        while let Some(prec) = self.get_precedence(self.peek_token()) {
            if prec < min_prec {
                break;
            }

            let op = self.peek_token().to_string();
            self.advance();

            let right = self.parse_binary_expr(prec + 1).map_err(|e| {
                format!(
                    "{}. Context: parsing binary expression with operator '{}'{}",
                    e,
                    op,
                    self.get_context()
                )
            })?;
            left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> Result<Expression, String> {
        let token = self.peek_token();

        if token == "(" {
            self.advance();

            // Check if this is a cast: (type)expression
            let saved_pos = self.pos;
            let is_cast = self.try_parse_cast();

            if is_cast {
                // This is a cast, parse the type
                self.pos = saved_pos;
                let cast_type = Box::new(self.parse_type()?);

                self.expect(")")?;

                // Parse the expression being cast
                let expr = Box::new(self.parse_primary_expr()?);
                eprintln!("[DEBUG] Parsed cast to type {:?}", cast_type);

                return Ok(Expression::Cast(cast_type, expr));
            } else {
                // Regular parenthesized expression
                let expr = self.parse_expr()?;

                self.expect(")")?;
                Ok(expr)
            }
        } else if token.starts_with("\"") || token.starts_with("'") {
            let literal = token.to_string();
            self.advance();
            Ok(Expression::Literal(literal))
        } else if token.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            let literal = token.to_string();
            self.advance();
            Ok(Expression::Literal(literal))
        } else if token == "!" || token == "-" || token == "~" {
            let op = token.to_string();
            self.advance();

            let expr = self.parse_primary_expr()?;
            Ok(Expression::UnaryOp(op, Box::new(expr)))
        } else {
            let var = token.to_string();
            self.advance();

            if self.peek_token() == "(" {
                self.advance();

                let mut args = Vec::new();
                while self.peek_token() != ")" && !self.is_eof() {
                    args.push(self.parse_expr()?);

                    if self.peek_token() == "," {
                        self.advance();
                    }
                }
                self.expect(")")?;
                Ok(Expression::FunctionCall(var, args))
            } else if self.peek_token() == "[" {
                self.advance();

                let index = self.parse_expr()?;

                self.expect("]")?;
                Ok(Expression::ArrayAccess(
                    Box::new(Expression::Identifier(var)),
                    Box::new(index),
                ))
            } else if self.peek_token() == "." {
                self.advance();

                let field = self.peek_token().to_string();
                self.advance();
                Ok(Expression::FieldAccess(
                    Box::new(Expression::Identifier(var)),
                    field,
                ))
            } else {
                Ok(Expression::Identifier(var))
            }
        }
    }

    fn try_parse_cast(&mut self) -> bool {
        // Try to determine if this is a cast by looking for type keywords
        let token = self.peek_token().to_lowercase();

        // Check if it's a known type
        if !self.is_type_start(&token) {
            return false;
        }

        // Save position and try to parse as a type
        let saved_pos = self.pos;

        // Try to parse the type
        if self.parse_type().is_err() {
            self.pos = saved_pos;
            return false;
        }

        // Check if followed by )
        let is_cast = self.peek_token() == ")";

        // Restore position
        self.pos = saved_pos;

        is_cast
    }

    fn parse_function_call(&mut self) -> Result<(), String> {
        let _name = self.peek_token().to_string();
        self.advance();

        if self.peek_token() == "(" {
            self.advance();
            let mut paren_depth = 1;
            while paren_depth > 0 && !self.is_eof() {
                match self.peek_token() {
                    "(" => paren_depth += 1,
                    ")" => paren_depth -= 1,
                    _ => {}
                }
                if paren_depth > 0 {
                    self.advance();
                }
            }
            self.advance();
        }

        if self.peek_token() == ";" {
            self.advance();
        }
        Ok(())
    }

    fn parse_literal(&mut self) -> Result<String, String> {
        let literal = self.peek_token().to_string();
        self.advance();

        Ok(literal)
    }

    fn get_precedence(&self, op: &str) -> Option<i32> {
        match op {
            "||" => Some(1),
            "&&" => Some(2),
            "|" => Some(3),
            "^" => Some(4),
            "&" => Some(5),
            "==" | "!=" => Some(6),
            "<" | ">" | "<=" | ">=" => Some(7),
            "+" | "-" => Some(8),
            "*" | "/" | "%" => Some(9),
            "=" => Some(0),
            _ => None,
        }
    }

    fn is_type_start(&self, token: &str) -> bool {
        let lower = token.to_lowercase();
        matches!(
            lower.as_str(),
            "char"
                | "uchar"
                | "short"
                | "ushort"
                | "int"
                | "uint"
                | "long"
                | "ulong"
                | "quad"
                | "uquad"
                | "float"
                | "double"
                | "unsigned"
                | "signed"
        )
    }

    fn skip_until(&mut self, end_marker: &str) -> Result<(), String> {
        while self.peek_token() != end_marker && !self.is_eof() {
            self.advance();
        }
        if self.peek_token() == end_marker {
            self.advance();
        }
        Ok(())
    }

    fn peek_token(&self) -> &str {
        self.tokens.get(self.pos).map(|s| s.as_str()).unwrap_or("")
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: &str) -> Result<(), String> {
        if self.is_eof() {
            return Err(format!(
                "Expected '{}' but reached end of file at line {}{}",
                expected,
                self.get_line_number(),
                self.get_context()
            ));
        }

        if self.peek_token().eq_ignore_ascii_case(expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected '{}' but found '{}' at token position {} (line {}){}",
                expected,
                self.peek_token(),
                self.pos,
                self.get_line_number(),
                self.get_context()
            ))
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
