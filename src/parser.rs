use crate::ast::*;
use std::collections::VecDeque;

pub struct Parser {
    tokens: VecDeque<TokenKind>,
    pos: usize,
    context_stack: Vec<String>, // Track nested contexts
}

impl Parser {
    pub fn new(content: &str) -> Self {
        let tokens = Self::tokenize(content);

        Parser {
            tokens,
            pos: 0,
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

    // fn get_context_stack(&self) -> String {
    //     if self.context_stack.is_empty() {
    //         "top-level".to_string()
    //     } else {
    //         self.context_stack.join(" > ")
    //     }
    // }

    fn get_context(&self) -> String {
        let mut context = "Context: ".to_string();
        for i in 0..=4 {
            if let Some(t) = self.tokens.get(self.pos + i - 2).map(|t| t.to_string()) {
                context.push_str(&t);
                context.push_str(" ");
            }
        }
        context
    }

    fn tokenize(content: &str) -> VecDeque<TokenKind> {
        let mut tokens = VecDeque::new();
        let mut current = String::new();
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '/' if chars.peek() == Some(&'/') => {
                    current.clear();
                    while let Some(c) = chars.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                '/' if chars.peek() == Some(&'*') => {
                    current.clear();
                    while let Some(c) = chars.next() {
                        if c == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
                ' ' | '\r' | '\n' | '\t' => {
                    if !current.is_empty() {
                        print!("{} ", current);
                        tokens.push_back(current.parse().unwrap());
                        current.clear();
                    }
                }
                '"' => {
                    if !current.is_empty() {
                        tokens.push_back(current.parse().unwrap());
                    }
                    current = ch.to_string();
                    while let Some(c) = chars.next() {
                        current.push(c);
                        if c != '\\' && chars.peek() == Some(&'"') {
                            current.push(chars.next().unwrap());
                            tokens.push_back(current.parse().unwrap());
                            current.clear();
                            break;
                        }
                    }
                }
                '\'' => {
                    if !current.is_empty() {
                        tokens.push_back(current.parse().unwrap());
                    }
                    current = ch.to_string();
                    while let Some(c) = chars.next() {
                        current.push(c);
                        if c != '\\' && chars.peek() == Some(&'\'') {
                            tokens.push_back(current.parse().unwrap());
                            current.push(chars.next().unwrap());
                            current.clear();
                            break;
                        }
                    }
                }
                _ if !current.is_empty()
                    && let TokenKind::Unknown(_) =
                        format!("{}{}", current, ch).parse().unwrap() =>
                {
                    print!("{} ", current);
                    tokens.push_back(current.parse().unwrap());
                    current = ch.to_string();
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push_back(current.parse().unwrap());
        }

        tokens
    }

    pub fn parse(&mut self) -> Result<Template, String> {
        let mut statements = Vec::new();
        self.push_context("top-level");

        while !self.is_eof() {
            statements.append(&mut self.parse_declaration()?);
        }

        self.pop_context();
        Ok(Template {
            statements,
            metadata: TemplateMetadata::default(),
        })
    }

    fn parse_declaration(&mut self) -> Result<Vec<Statement>, String> {
        match self.peek_token()? {
            // kinda cursed, but this is how it is
            TokenKind::Keyword(Keyword::Struct) => {
                if let Ok(d) = self.try_parse_var_decl(false) {
                    Ok(d)
                } else {
                    Ok(vec![Statement::StructDef(self.parse_struct()?)])
                }
            }
            TokenKind::Keyword(Keyword::Enum) => self.parse_enum().map(|e| vec![e]),
            TokenKind::Keyword(Keyword::Typedef) => self.parse_typedef().map(|e| vec![e]),
            TokenKind::Keyword(Keyword::Local) => self.parse_var_decl(true),
            TokenKind::Keyword(Keyword::DataType(_) | Keyword::Unsigned | Keyword::Signed) => {
                self.parse_var_decl(false)
            }
            TokenKind::Ident(_) if matches!(self.peek_token_after(1)?, TokenKind::Ident(_)) => {
                self.parse_var_decl(false)
            }
            _ => self.parse_statement().map(|e| vec![e]),
        }
    }

    // fn is_type(&self, token: &str) -> bool {
    //     !matches!(
    //         token,
    //         "auto"
    //             | "break"
    //             | "case"
    //             | "const"
    //             | "continue"
    //             | "default"
    //             | "else"
    //             | "enum"
    //             | "false"
    //             | "for"
    //             | "if"
    //             | "include"
    //             | "return"
    //             | "static"
    //             | "struct"
    //             | "switch"
    //             | "true"
    //             | "typedef"
    //             | "union"
    //             | "void"
    //             | "while"
    //     ) && !token.chars().nth(0).is_some_and(|c| char::is_numeric(c))
    // }

    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expr()?;

        if self.peek_token()? == &Punctuator::Assign {
            self.advance();
            let rhs = self.parse_expr()?;
            self.expect(Punctuator::Semicolon)?;
            return Ok(Statement::Assign {
                left: expr,
                right: rhs,
            });
        }

        self.expect(Punctuator::Semicolon)?;
        Ok(Statement::Expr(expr))
    }

    fn parse_typedef(&mut self) -> Result<Statement, String> {
        self.expect(Keyword::Typedef)?;

        let base_ty = self
            .parse_type()
            .map_err(|e| format!("{}. Context: parsing typedef{}", e, self.get_context()))?;

        eprintln!("[DEBUG] Parsing typedef type alias");

        let name = self.peek_token()?.to_string();
        eprintln!("[DEBUG] Typedef name: {}", name);
        self.advance();

        let mut ty = base_ty;
        while self.peek_token()? == &Punctuator::LBracket {
            self.advance();

            let size = if self.peek_token()? != &Punctuator::RBracket {
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

            self.expect(Punctuator::RBracket).map_err(|e| {
                format!(
                    "{}. Context: in typedef array definition{}",
                    e,
                    self.get_context()
                )
            })?;

            ty = DataType::Array(Box::new(ty), size);
        }

        if self.peek_token()? == &Punctuator::LAngledBracket {
            self.skip_until(Punctuator::RAngledBracket)?;
        }

        self.expect(Punctuator::Semicolon)
                .map_err(|e| format!("{}. Context: at end of typedef statement. Current token: '{}' at position {}. Expected ';'", e, self.peek_token().unwrap(), self.pos))?;

        Ok(Statement::TypeDef { ident: name, ty })
    }

    fn parse_struct(&mut self) -> Result<Struct, String> {
        self.expect(Keyword::Struct)?;

        let ident = if let TokenKind::Ident(i) = self.peek_token()? {
            Some(i.to_owned())
        } else {
            None
        };
        if ident.is_some() {
            self.advance();
        }
        eprintln!("[DEBUG] Parsing struct: {:?}", ident);

        if self.peek_token()? == &Punctuator::LAngledBracket {
            self.skip_until(Punctuator::RAngledBracket)?;
        }

        self.expect(Punctuator::LBrace)?;
        eprintln!("[DEBUG] Entered struct body");

        self.push_context(&format!("struct {:?} fields", ident));
        let fields = self.parse_struct_body()?;
        self.pop_context();

        self.expect(Punctuator::RBrace)?;
        eprintln!("[DEBUG] Struct {:?} closed successfully", ident);

        // Handle optional style annotation
        if self.peek_token()? == &Punctuator::LAngledBracket {
            self.skip_until(Punctuator::RAngledBracket)?;
        }

        // Optional type name after closing brace
        // if self.peek_token()? != &Punctuator::Semicolon
        //     && self.peek_token()? != &Keyword::Local
        //     && !self.is_type_start(self.peek_token()?)
        // {
        //     self.advance();
        // }

        // if self.peek_token()? == &Punctuator::Semicolon {
        //     self.advance();
        // }

        Ok(Struct {
            ident,
            body: fields,
        })
    }

    fn parse_struct_body(&mut self) -> Result<Vec<StructItem>, String> {
        let mut items = Vec::new();

        while self.peek_token()? != &Punctuator::RBrace {
            let decl = self.parse_declaration()?;
            for d in decl {
                items.push(self.into_struct_item(d)?);
            }
        }

        Ok(items)
    }

    fn into_struct_item(&self, decl: Statement) -> Result<StructItem, String> {
        match decl {
            Statement::VarDecl {
                ident,
                ty,
                value: None,
                local: false,
            } => Ok(StructItem::Field(StructField { ident, ty })),
            _ => Ok(StructItem::Statement(Box::new(decl))),
        }
    }

    fn parse_enum(&mut self) -> Result<Statement, String> {
        self.expect(Keyword::Enum)?;

        let ty = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let ident = self
            .peek_token()?
            .ident()
            .ok_or_else(|| format!("wanted ident, got {}", self.peek_token().unwrap()))?
            .to_string();
        eprintln!("[DEBUG] Parsing enum: {}", ident);
        self.advance();

        if self.peek_token()? == &Punctuator::Colon {
            self.advance();

            self.parse_type()?;
        }

        self.expect(Punctuator::LBrace)?;

        let mut variants = Vec::new();

        while self.peek_token()? != &Punctuator::RBrace {
            let variant_name = self
                .peek_token()?
                .ident()
                .ok_or_else(|| format!("wanted ident, got {}", self.peek_token().unwrap()))?
                .to_string();
            eprintln!("[DEBUG] Enum variant: {}", variant_name);
            self.advance();

            let value = if self.peek_token()? == &Punctuator::Assign {
                self.advance();

                Some(self.parse_literal()?.parse::<i64>().unwrap_or(0))
            } else {
                None
            };

            variants.push((variant_name, value));

            if self.peek_token()? == &Punctuator::Comma {
                self.advance();
            }
        }

        self.expect(Punctuator::RBrace)?;
        if self.peek_token()? == &Punctuator::LAngledBracket {
            self.skip_until(Punctuator::RAngledBracket)?;
        }
        self.expect(Punctuator::Semicolon)?;

        Ok(Statement::EnumDef(Enum {
            ident,
            ty,
            variants,
        }))
    }

    fn parse_var_decl(&mut self, local: bool) -> Result<Vec<Statement>, String> {
        let mut stmts: Vec<Statement> = vec![];
        if local {
            self.expect(Keyword::Local)?;
        }

        let base_ty = self.parse_type().map_err(|e| {
            format!(
                "{}. Context: parsing variable type{}",
                e,
                self.get_context()
            )
        })?;

        loop {
            let ident = self
                .peek_token()?
                .ident()
                .ok_or_else(|| format!("wanted ident, got {}", self.peek_token().unwrap()))?
                .to_string();
            eprintln!("[DEBUG] Variable name: {}", ident);
            self.advance();

            let mut ty = base_ty.clone();
            while self.peek_token()? == &Punctuator::LBracket {
                self.advance();

                let size = if self.peek_token()? != &Punctuator::RBracket {
                    Some(Box::new(self.parse_expr().map_err(|e| {
                        format!(
                            "{}. Context: parsing array size for variable '{}'{}",
                            e,
                            ident,
                            self.get_context()
                        )
                    })?))
                } else {
                    None
                };

                self.expect(Punctuator::RBracket)?;

                ty = DataType::Array(Box::new(ty), size);
            }

            let value = if self.peek_token()? == &Punctuator::Assign {
                self.advance();

                Some(self.parse_expr().map_err(|e| {
                    format!(
                        "{}. Context: parsing initialization value for variable '{}'{}",
                        e,
                        ident,
                        self.get_context()
                    )
                })?)
            } else {
                None
            };

            let stmt = Statement::VarDecl {
                ident,
                ty,
                value,
                local,
            };
            stmts.push(stmt);

            if self.peek_token()? != &Punctuator::Comma {
                break;
            }
        }

        if self.peek_token()? == &Punctuator::LAngledBracket {
            self.skip_until(Punctuator::RAngledBracket)?;
        }

        if self.peek_token()? == &Punctuator::Semicolon {
            self.advance();
        }

        Ok(stmts)
    }

    fn try_parse_var_decl(&mut self, local: bool) -> Result<Vec<Statement>, String> {
        let pos = self.pos;
        match self.parse_var_decl(local) {
            Ok(d) => Ok(d),
            Err(e) => {
                self.pos = pos;
                Err(e)
            }
        }
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect(Keyword::If)?;
        self.expect(Punctuator::LParen)?;

        eprintln!("[DEBUG] Parsing if condition");
        let condition = self
            .parse_expr()
            .map_err(|e| format!("{}. Context: parsing if condition{}", e, self.get_context()))?;

        self.expect(Punctuator::RParen)?;

        let then_block = if self.peek_token()? == &Punctuator::LBrace {
            eprintln!("[DEBUG] Entering if block");
            self.advance();

            self.push_context("if block");
            let block = self.parse_block()?;
            self.pop_context();

            self.expect(Punctuator::RBrace)?;
            block
        } else {
            eprintln!("[DEBUG] Entering one-line if");
            // self.advance();

            self.parse_declaration()?
        };

        let else_block = if self.peek_token()? == &Keyword::Else {
            self.advance();

            if self.peek_token()? == &Punctuator::LBrace {
                eprintln!("[DEBUG] Entering else block");
                self.advance();

                self.push_context("else block");
                let block = self.parse_block()?;
                self.pop_context();

                self.expect(Punctuator::RBrace)?;
                Some(block)
            } else {
                Some(self.parse_declaration()?)
            }
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block: Block(then_block),
            else_block: else_block.map(|b| Block(b)),
        })
    }

    fn parse_while(&mut self) -> Result<Statement, String> {
        self.expect(Keyword::While)?;

        self.expect(Punctuator::LParen)?;

        eprintln!("[DEBUG] Parsing while condition");
        let condition = self.parse_expr().map_err(|e| {
            format!(
                "{}. Context: parsing while condition{}",
                e,
                self.get_context()
            )
        })?;

        self.expect(Punctuator::RParen)?;

        let body = if self.peek_token()? == &Punctuator::LBrace {
            eprintln!("[DEBUG] Entering while block");
            self.advance();

            self.push_context("while block");
            let block = self.parse_block()?;
            self.pop_context();

            eprintln!(
                "[DEBUG] About to expect closing brace, current token: '{}'",
                self.peek_token()?
            );
            self.expect(Punctuator::RBrace)?;
            eprintln!("[DEBUG] Exiting while block");
            block
        } else {
            vec![self.parse_statement()?]
        };

        Ok(Statement::While {
            condition,
            body: Block(body),
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();

        while self.peek_token()? != &Punctuator::RBrace {
            stmts.append(&mut self.parse_declaration()?);
        }

        Ok(stmts)
    }

    fn parse_switch(&mut self) -> Result<Statement, String> {
        self.expect(Keyword::Switch)?;

        self.expect(Punctuator::LParen)?;

        eprintln!("[DEBUG] Parsing switch expression");
        let expr = self.parse_expr()?;

        self.expect(Punctuator::RParen)?;
        self.expect(Punctuator::LBrace)?;

        eprintln!("[DEBUG] Entering switch block");
        let mut cases = Vec::new();

        while self.peek_token()? != &Punctuator::RBrace {
            // not really sure what to do in this case except clone (either way it's cheap)
            let token = self.peek_token()?.clone();

            if token == Keyword::Case || token == Keyword::Default {
                if token == Keyword::Case {
                    eprintln!("[DEBUG] Parsing case label");
                    self.advance();

                    // Skip the case value
                    while self.peek_token()? != &Punctuator::Colon {
                        self.advance();
                    }
                } else {
                    eprintln!("[DEBUG] Parsing default label");
                    self.advance();
                }

                if self.peek_token()? == &Punctuator::Colon {
                    self.advance();
                }

                let mut case_body = Vec::new();
                while self.peek_token()? != &Keyword::Case
                    && self.peek_token()? != &Keyword::Default
                    && self.peek_token()? != &Punctuator::RBrace
                {
                    if self.peek_token()? == &Keyword::Case
                        || self.peek_token()? == &Keyword::Default
                        || self.peek_token()? == &Punctuator::RBrace
                    {
                        break;
                    }
                    case_body.append(&mut self.parse_declaration()?);
                }
                cases.push(Block(case_body));
            } else {
                break;
            }
        }

        self.expect(Punctuator::RBrace)?;
        eprintln!("[DEBUG] Exiting switch block");

        Ok(Statement::Switch { expr, cases })
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        eprintln!("[DEBUG] Parsing return statement");
        self.advance();

        let value = if self.peek_token()? != &Punctuator::Semicolon {
            Some(self.parse_expr().map_err(|e| {
                format!("{}. Context: parsing return value{}", e, self.get_context())
            })?)
        } else {
            None
        };

        if self.peek_token()? == &Punctuator::Semicolon {
            self.advance();
        }
        Ok(Statement::Return(value))
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let token = self.peek_token()?;
        eprintln!("[DEBUG] parse_statement: token = '{}'", token);

        match token {
            TokenKind::Keyword(Keyword::If) => self.parse_if(),
            TokenKind::Keyword(Keyword::While) => self.parse_while(),
            TokenKind::Keyword(Keyword::Switch) => self.parse_switch(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Punc(Punctuator::LBrace) => {
                self.advance();
                let block = self.parse_block()?;
                self.expect(Punctuator::RBrace)?;
                Ok(Statement::Block(Block(block)))
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_type(&mut self) -> Result<DataType, String> {
        match self.peek_token()? {
            TokenKind::Keyword(Keyword::Struct) => {
                let s = self.parse_struct()?;
                Ok(DataType::Struct(s))
            }

            _ => self.parse_basic_type(),
        }
    }

    fn parse_basic_type(&mut self) -> Result<DataType, String> {
        let token = self.peek_token()?;

        if token == &Keyword::Unsigned {
            self.advance();

            let next_token = self.peek_token()?;
            let base_type = match next_token {
                TokenKind::Keyword(Keyword::DataType(dt)) => {
                    let d = dt.clone();
                    self.advance();
                    match self.peek_token()? {
                        TokenKind::Keyword(Keyword::DataType(second)) => {
                            if second == &DataType::I32 {
                                self.advance();
                                DataType::U64
                            } else {
                                DataType::U32
                            }
                        }
                        _ => d.into_unsigned(),
                    }
                }
                _ => {
                    return Err("no type after 'unsigned' found".to_string());
                }
            };
            return Ok(base_type);
        }

        if token == &Keyword::Signed {
            self.advance();

            let next_token = self.peek_token()?;
            let base_type = match next_token {
                TokenKind::Keyword(Keyword::DataType(dt)) => {
                    let d = dt.clone();
                    self.advance();
                    match self.peek_token()? {
                        TokenKind::Keyword(Keyword::DataType(second)) => {
                            if second == &DataType::I32 {
                                self.advance();
                                DataType::I64
                            } else {
                                DataType::I32
                            }
                        }
                        _ => d.into_signed(),
                    }
                }
                _ => {
                    return Err("no type after 'signed' found".to_string());
                }
            };
            return Ok(base_type);
        }

        let base_type = match token {
            TokenKind::Keyword(Keyword::DataType(dt)) => dt.clone(),
            TokenKind::Ident(i) => DataType::Custom(i.clone()),
            t => return Err(format!("Nonsense datatype {}", t)),
        };

        self.advance();

        Ok(base_type)
    }

    fn parse_expr(&mut self) -> Result<Expression, String> {
        self.parse_ternary_expr()
    }

    fn parse_ternary_expr(&mut self) -> Result<Expression, String> {
        let expr = self.parse_binary_expr(0)?;

        if self.peek_token()? == &Punctuator::Question {
            self.advance();

            let true_expr = self.parse_expr()?;

            self.expect(Punctuator::Colon)?;

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

        while let TokenKind::Punc(p) = self.peek_token()?
            && let Some(prec) = self.get_precedence(p)
        {
            if prec < min_prec {
                break;
            }

            let op = p.to_string();
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
        let token = self.peek_token()?;

        match token {
            TokenKind::Punc(Punctuator::LParen) => {
                self.advance();

                let saved_pos = self.pos;
                let is_cast = self.is_cast();

                if is_cast {
                    self.pos = saved_pos;
                    let cast_type = Box::new(self.parse_type()?);

                    self.expect(Punctuator::RParen)?;

                    // Parse the expression being cast
                    let expr = Box::new(self.parse_primary_expr()?);
                    eprintln!("[DEBUG] Parsed cast to type {:?}", cast_type);

                    return Ok(Expression::Cast(cast_type, expr));
                } else {
                    // Regular parenthesized expression
                    let expr = self.parse_expr()?;

                    self.expect(Punctuator::RParen)?;
                    Ok(expr)
                }
            }
            TokenKind::Char(c) => {
                let char = c.to_string();
                self.advance();
                Ok(Expression::Literal(char))
            }
            TokenKind::String(s) => {
                let str = s.to_string();
                self.advance();
                Ok(Expression::Literal(str))
            }
            TokenKind::Int(i) => {
                let int = i.to_string();
                self.advance();
                Ok(Expression::Literal(int))
            }
            TokenKind::Float(f) => {
                let float = f.to_string();
                self.advance();
                Ok(Expression::Literal(float))
            }
            TokenKind::Punc(
                Punctuator::Plus | Punctuator::Minus | Punctuator::BitNot | Punctuator::Not,
            ) => {
                let p = token.to_string();
                self.advance();
                let expr = self.parse_primary_expr()?;
                Ok(Expression::UnaryOp(p, Box::new(expr)))
            }
            _ => {
                let var = token.to_string();
                self.advance();

                if self.peek_token()? == &Punctuator::LParen {
                    self.advance();

                    let mut args = Vec::new();
                    while self.peek_token()? != &Punctuator::RParen && !self.is_eof() {
                        args.push(self.parse_expr()?);

                        if self.peek_token()? == &Punctuator::Comma {
                            self.advance();
                        }
                    }
                    self.expect(Punctuator::RParen)?;
                    Ok(Expression::Call(var, args))
                } else if self.peek_token()? == &Punctuator::LBracket {
                    self.advance();

                    let index = self.parse_expr()?;

                    self.expect(Punctuator::RBracket)?;
                    Ok(Expression::ArrayAccess(
                        Box::new(Expression::Identifier(var)),
                        Box::new(index),
                    ))
                } else if self.peek_token()? == &Punctuator::Dot {
                    self.advance();

                    let field = self.peek_token()?.to_string();
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
    }

    fn is_cast(&mut self) -> bool {
        // Try to determine if this is a cast by looking for type keywords
        let token = if let Ok(t) = self.peek_token() {
            t
        } else {
            return false;
        };

        // Check if it's a known type
        if !matches!(
            token,
            TokenKind::Keyword(Keyword::DataType(_)) | TokenKind::Ident(_)
        ) {
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
        let is_cast = self.peek_token().unwrap() == &Punctuator::RParen;

        // Restore position
        self.pos = saved_pos;

        is_cast
    }

    // fn parse_function_call(&mut self) -> Result<(), String> {
    //     let _name = self.peek_token()?.to_string();
    //     self.advance();

    //     if self.peek_token()? == &Punctuator::LParen {
    //         self.advance();
    //         let mut paren_depth = 1;
    //         while paren_depth > 0 && !self.is_eof() {
    //             match self.peek_token()? {
    //                 Punctuator::LParen => paren_depth += 1,
    //                 Punctuator::RParen => paren_depth -= 1,
    //                 _ => {}
    //             }
    //             if paren_depth > 0 {
    //                 self.advance();
    //             }
    //         }
    //         self.advance();
    //     }

    //     if self.peek_token()? == &Punctuator::Semicolon {
    //         self.advance();
    //     }
    //     Ok(())
    // }

    fn parse_literal(&mut self) -> Result<String, String> {
        let literal = self.peek_token()?.to_string();
        self.advance();

        Ok(literal)
    }

    fn get_precedence(&self, op: &Punctuator) -> Option<i32> {
        match op {
            Punctuator::Or => Some(1),
            Punctuator::And => Some(2),
            Punctuator::BitOr => Some(3),
            Punctuator::BitXor => Some(4),
            Punctuator::Ampersand => Some(5),
            Punctuator::Equal | Punctuator::NotEqual => Some(6),
            Punctuator::LAngledBracket
            | Punctuator::RAngledBracket
            | Punctuator::LessEqual
            | Punctuator::GreaterEqual => Some(7),
            Punctuator::Plus | Punctuator::Minus => Some(8),
            Punctuator::Asterisk | Punctuator::Div | Punctuator::Mod => Some(9),
            Punctuator::Assign => Some(0),
            _ => None,
        }
    }

    fn is_type_start(&self, token: &TokenKind) -> bool {
        matches!(token, TokenKind::Keyword(Keyword::DataType(_)))
    }

    fn skip_until<T: Into<TokenKind>>(&mut self, end_marker: T) -> Result<(), String> {
        let end = end_marker.into();
        while self.peek_token()? != &end && !self.is_eof() {
            self.advance();
        }
        if self.peek_token()? == &end {
            self.advance();
        }
        Ok(())
    }

    fn peek_token(&self) -> Result<&TokenKind, String> {
        self.peek_token_after(0)
    }

    fn peek_token_after(&self, after: usize) -> Result<&TokenKind, String> {
        self.tokens
            .get(self.pos + after)
            .ok_or_else(|| format!("EOF at position {}", self.pos))
    }

    fn advance(&mut self) {
        if !self.is_eof() {
            self.pos += 1;
        }
    }

    fn expect<T>(&mut self, expected: T) -> Result<(), String>
    where
        T: Into<TokenKind>,
    {
        let token: TokenKind = expected.into();
        if self.is_eof() {
            return Err(format!(
                "Expected '{}' but reached end of file {}",
                token,
                self.get_context()
            ));
        }

        if self.peek_token()? == &token {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected '{}' but found {:?} '{}' at token position {} {}",
                token,
                self.peek_token()?,
                self.peek_token()?,
                self.pos,
                self.get_context()
            ))
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
