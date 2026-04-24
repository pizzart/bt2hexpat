use crate::ast::{
    attr::{Attribute, AttributeType, Attributes},
    data_type::*,
    stmt::*,
    template::*,
    token::*,
};
use std::collections::VecDeque;

pub struct Parser {
    tokens: VecDeque<(TokenKind, String)>,
    pos: usize,
}

impl Parser {
    pub fn new(content: &str) -> Self {
        let tokens = Self::tokenize(content);
        Parser { tokens, pos: 0 }
    }

    fn get_context(&self) -> String {
        let mut context = "Context: ".to_string();
        for i in 0..=4 {
            if let Some(t) = self.tokens.get(self.pos + i - 2).map(|(_, s)| s) {
                context.push_str(&t);
                context.push_str(" ");
            }
        }
        context
    }

    fn tokenize(content: &str) -> VecDeque<(TokenKind, String)> {
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
                '"' => {
                    if !current.is_empty() {
                        tokens.push_back((current.parse().unwrap(), current));
                    }
                    current = ch.to_string();
                    while let Some(c) = chars.next() {
                        current.push(c);
                        if c == '"' && current.len() == 2 {
                            tokens
                                .push_back((current.parse().unwrap(), current.drain(..).collect()));
                            break;
                        }
                        if c != '\\' && chars.peek() == Some(&'"') {
                            current.push(chars.next().unwrap());
                            tokens
                                .push_back((current.parse().unwrap(), current.drain(..).collect()));
                            break;
                        }
                    }
                }
                '\'' => {
                    if !current.is_empty() {
                        tokens.push_back((current.parse().unwrap(), current));
                    }
                    current = ch.to_string();
                    while let Some(c) = chars.next() {
                        current.push(c);
                        if c != '\\' && chars.peek() == Some(&'\'') {
                            tokens
                                .push_back((current.parse().unwrap(), current.drain(..).collect()));
                            current.push(chars.next().unwrap());
                            break;
                        }
                    }
                }
                _ if ch.is_whitespace() => {
                    if !current.is_empty() {
                        tokens.push_back((current.parse().unwrap(), current.drain(..).collect()));
                    }
                }
                _ if !current.is_empty()
                    && let TokenKind::Unknown(_) =
                        format!("{}{}", current, ch).parse().unwrap() =>
                {
                    tokens.push_back((current.parse().unwrap(), current.drain(..).collect()));
                    current = ch.to_string();
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push_back((current.parse().unwrap(), current));
        }

        tokens
    }

    pub fn parse(&mut self) -> Result<Template, String> {
        let mut statements = Vec::new();

        while !self.is_eof() {
            statements.append(&mut self.parse_def_or_stmt()?);
        }

        Ok(Template {
            statements,
            metadata: TemplateMetadata::default(),
        })
    }

    fn parse_def_or_stmt(&mut self) -> Result<Vec<Statement>, String> {
        match self.peek_token()? {
            // kinda cursed, but this is how it is
            TokenKind::Keyword(Keyword::Struct | Keyword::Union) => {
                if let Ok(d) = self.try_parse_var_decl(false) {
                    Ok(d)
                } else {
                    Ok(vec![Statement::StructDef(self.parse_struct()?)])
                }
            }
            TokenKind::Keyword(Keyword::Enum) => {
                if let Ok(d) = self.try_parse_var_decl(false) {
                    Ok(d)
                } else {
                    Ok(vec![Statement::EnumDef(self.parse_enum()?)])
                }
            }
            TokenKind::Keyword(Keyword::Typedef) => self.parse_typedef().map(|e| vec![e]),
            TokenKind::Keyword(Keyword::Local) => self.parse_var_decl(true),
            TokenKind::Keyword(Keyword::DataType(_) | Keyword::Unsigned | Keyword::Signed) => {
                self.parse_var_or_fn_decl()
            }
            TokenKind::Ident(_) if matches!(self.peek_token_after(1)?, TokenKind::Ident(_)) => {
                self.parse_var_or_fn_decl()
            }
            _ => self.parse_stmt().map(|e| vec![e]),
        }
    }

    fn parse_expr_stmt(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expr()?;

        let s = self.peek_token()?;
        if s.is_assign_op() {
            let sign = s.to_string();
            self.advance();
            let rhs = self.parse_expr()?;
            self.expect(Punctuator::Semicolon)?;
            return Ok(Statement::Assign {
                left: expr,
                sign,
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

        let attrs = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.parse_attrs()?
        } else {
            Attributes(vec![])
        };

        self.expect(Punctuator::Semicolon)
                .map_err(|e| format!("{}. Context: at end of typedef statement. Current token: '{}' at position {}. Expected ';'", e, self.peek_token().unwrap(), self.pos))?;

        Ok(Statement::TypeDef {
            ident: name,
            ty,
            attrs,
        })
    }

    fn parse_struct(&mut self) -> Result<Struct, String> {
        let keyword = self.expect_any(vec![Keyword::Struct, Keyword::Union])?;
        let ty = match keyword {
            TokenKind::Keyword(Keyword::Struct) => StructType::Struct,
            TokenKind::Keyword(Keyword::Union) => StructType::Union,
            _ => panic!(),
        };

        let ident = if let TokenKind::Ident(i) = self.peek_token()? {
            Some(i.to_owned())
        } else {
            None
        };
        if ident.is_some() {
            self.advance();
        }
        eprintln!("[DEBUG] Parsing struct: {:?}", ident);

        if self.peek_token()? == &Punctuator::Semicolon {
            self.advance();
            return Ok(Struct {
                ty,
                ident,
                body: Block(vec![]),
                attrs: Attributes(vec![]),
            });
        }

        if self.peek_token()? == &Punctuator::LAngledBracket {
            self.skip_until(Punctuator::RAngledBracket)?;
        }

        self.expect(Punctuator::LBrace)?;
        eprintln!("[DEBUG] Entered struct body");

        let fields = self.parse_struct_body()?;

        self.expect(Punctuator::RBrace)?;
        eprintln!("[DEBUG] Struct {:?} closed successfully", ident);

        let attrs = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.parse_attrs()?
        } else {
            Attributes(vec![])
        };

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
            ty,
            ident,
            body: fields,
            attrs,
        })
    }

    fn parse_struct_body(&mut self) -> Result<Block, String> {
        let mut items = Vec::new();

        while self.peek_token()? != &Punctuator::RBrace {
            let decl = self.parse_def_or_stmt()?;
            for d in decl {
                items.push(d);
            }
        }

        Ok(Block(items))
    }

    fn parse_attrs(&mut self) -> Result<Attributes, String> {
        self.expect(Punctuator::LAngledBracket)?;
        let mut attrs = vec![];
        while self.peek_token()? != &Punctuator::RAngledBracket {
            let ty = match self.peek_token()?.as_attribute() {
                Some(attr) => attr.clone(),
                _ => return Err(format!("unknown attribute name {}", self.peek_token()?)),
            };
            self.advance();
            self.expect(Punctuator::Assign)?;
            let value = self.parse_primary_expr()?;
            attrs.push(Attribute { ty, value });
            if self.peek_token()? == &Punctuator::Comma {
                self.advance();
            }
        }
        self.expect(Punctuator::RAngledBracket)?;
        Ok(Attributes(attrs))
    }

    fn attrs_get_pos(&self, attrs: &Attributes) -> Option<Expression> {
        attrs
            .0
            .iter()
            .filter(|a| matches!(a.ty, AttributeType::Pos))
            .next()
            .map(|a| a.value.clone())
    }

    fn parse_enum(&mut self) -> Result<Enum, String> {
        self.expect(Keyword::Enum)?;

        let ty = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let ident = self.peek_token()?.ident().map(|s| s.to_owned());
        eprintln!("[DEBUG] Parsing enum: {:?}", ident);
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
                Some(Box::new(self.parse_literal()?))
            } else {
                None
            };

            variants.push((variant_name, value));

            if self.peek_token()? == &Punctuator::Comma {
                self.advance();
            }
        }

        self.expect(Punctuator::RBrace)?;

        let attrs = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.parse_attrs()?
        } else {
            Attributes(vec![])
        };

        if self.peek_token()? == &Punctuator::Semicolon {
            self.advance();
        }
        // self.expect(Punctuator::Semicolon)?;

        Ok(Enum {
            ident,
            ty,
            variants,
            attrs,
        })
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        let ident = self
            .peek_token()?
            .ident()
            .ok_or_else(|| format!("wanted ident, got {}", self.peek_token().unwrap()))?
            .to_string();
        self.advance();
        Ok(ident)
    }

    fn parse_var_or_fn_decl(&mut self) -> Result<Vec<Statement>, String> {
        let pos = self.pos;
        match self.parse_var_decl(false) {
            Ok(d) => Ok(d),
            Err(e) => {
                dbg!(e);
                self.pos = pos;
                self.parse_fn_decl().map(|s| vec![s])
            }
        }
    }

    fn parse_fn_decl(&mut self) -> Result<Statement, String> {
        let ty = self.parse_type().map_err(|e| {
            format!(
                "{}. Context: parsing variable type{}",
                e,
                self.get_context()
            )
        })?;

        let ident = self.parse_ident()?;
        self.expect(Punctuator::LParen)?;
        let mut args = vec![];
        while self.peek_token()? != &Punctuator::RParen {
            let data_type = self.parse_type().map_err(|e| {
                format!(
                    "{}. Context: parsing variable type{}",
                    e,
                    self.get_context()
                )
            })?;
            let ident = self.parse_ident()?;
            args.push((data_type, ident));
            if self.peek_token()? == &Punctuator::Comma {
                self.advance();
            }
        }
        self.expect(Punctuator::RParen)?;
        self.expect(Punctuator::LBrace)?;
        let block = self.parse_block()?;
        self.expect(Punctuator::RBrace)?;
        Ok(Statement::FnDef {
            ty,
            ident,
            args: Args(args),
            block: Block(block),
        })
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
            let ident = self.parse_ident()?;

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

            let bits = if self.peek_token()? == &Punctuator::Colon {
                self.advance();

                match self.parse_literal()? {
                    Expression::Literal(l) => l.int().cloned(),
                    _ => None,
                }
            } else {
                None
            };

            let attrs = if self.peek_token()? == &Punctuator::LAngledBracket {
                self.parse_attrs()?
            } else {
                Attributes(vec![])
            };

            let pos = self.attrs_get_pos(&attrs);

            let stmt = Statement::VarDef {
                ident,
                ty,
                value,
                local,
                bits,
                pos,
                attrs,
            };
            stmts.push(stmt);

            if self.peek_token()? != &Punctuator::Comma {
                break;
            }
        }

        self.expect(Punctuator::Semicolon)?;

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

            let block = self.parse_block()?;

            self.expect(Punctuator::RBrace)?;
            block
        } else {
            eprintln!("[DEBUG] Entering one-line if");
            // self.advance();

            self.parse_def_or_stmt()?
        };

        let else_block = if self.peek_token()? == &Keyword::Else {
            self.advance();

            if self.peek_token()? == &Punctuator::LBrace {
                eprintln!("[DEBUG] Entering else block");
                self.advance();

                let block = self.parse_block()?;

                self.expect(Punctuator::RBrace)?;
                Some(block)
            } else {
                Some(self.parse_def_or_stmt()?)
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

            let block = self.parse_block()?;

            eprintln!(
                "[DEBUG] About to expect closing brace, current token: '{}'",
                self.peek_token()?
            );
            self.expect(Punctuator::RBrace)?;
            eprintln!("[DEBUG] Exiting while block");
            block
        } else {
            vec![self.parse_stmt()?]
        };

        Ok(Statement::While {
            condition,
            body: Block(body),
        })
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();

        while self.peek_token()? != &Punctuator::RBrace {
            stmts.append(&mut self.parse_def_or_stmt()?);
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
        let mut default = None;

        while self.peek_token()? != &Punctuator::RBrace {
            // not really sure what to do in this case except clone (either way it's cheap)
            let token = self.peek_token()?;

            match token {
                TokenKind::Keyword(Keyword::Case) => {
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(Punctuator::Colon)?;
                    let mut stmts = vec![];
                    while self.peek_token()? != &Keyword::Break {
                        stmts.append(&mut self.parse_def_or_stmt()?);
                    }
                    self.expect(Keyword::Break)?;
                    self.expect(Punctuator::Semicolon)?;
                    cases.push((expr, Block(stmts)));
                }
                TokenKind::Keyword(Keyword::Default) => {
                    self.advance();
                    self.expect(Punctuator::Colon)?;
                    let mut stmts = vec![];
                    while self.peek_token()? != &Punctuator::RBrace {
                        stmts.append(&mut self.parse_def_or_stmt()?);
                    }
                    default.replace(Block(stmts));
                }
                _ => {
                    return Err(format!(
                        "invalid token encountered in switch statement, expected case or default, got {}",
                        token
                    ));
                }
            }
        }

        self.expect(Punctuator::RBrace)?;
        eprintln!("[DEBUG] Exiting switch block");

        Ok(Statement::Switch {
            expr,
            cases,
            default,
        })
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

    fn parse_stmt(&mut self) -> Result<Statement, String> {
        let token = self.peek_token()?;
        eprintln!("[DEBUG] parse_statement: token = '{}'", token);

        match token {
            TokenKind::Keyword(Keyword::If) => self.parse_if(),
            TokenKind::Keyword(Keyword::While) => self.parse_while(),
            TokenKind::Keyword(Keyword::Switch) => self.parse_switch(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Keyword(Keyword::Break) => {
                self.expect(Keyword::Break)?;
                self.advance();
                Ok(Statement::Break)
            }
            TokenKind::Keyword(Keyword::Continue) => {
                self.expect(Keyword::Continue)?;
                self.advance();
                Ok(Statement::Continue)
            }
            TokenKind::Punc(Punctuator::LBrace) => {
                self.advance();
                let block = self.parse_block()?;
                self.expect(Punctuator::RBrace)?;
                Ok(Statement::Block(Block(block)))
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_type(&mut self) -> Result<DataType, String> {
        match self.peek_token()? {
            TokenKind::Keyword(Keyword::Struct) => {
                let s = self.parse_struct()?;
                Ok(DataType::Struct(s))
            }
            TokenKind::Keyword(Keyword::Enum) => {
                let e = self.parse_enum()?;
                Ok(DataType::Enum(Box::new(e)))
            }

            _ => self.parse_basic_type(),
        }
    }

    fn parse_basic_type(&mut self) -> Result<DataType, String> {
        let token = self.peek_token()?.clone();

        let base_type = match token {
            TokenKind::Keyword(Keyword::Unsigned | Keyword::Signed) => {
                self.advance();

                let next_token = self.peek_token()?;
                match next_token {
                    TokenKind::Keyword(Keyword::DataType(dt)) => {
                        let d = dt.clone();
                        if token == Keyword::Unsigned {
                            d.to_unsigned()
                        } else {
                            d.to_signed()
                        }
                    }
                    _ => {
                        return Err("no type after 'unsigned' found".to_string());
                    }
                }
            }
            TokenKind::Keyword(Keyword::DataType(dt)) => dt,
            TokenKind::Ident(i) => DataType::Custom(i),
            t => return Err(format!("Nonsense datatype {}", t)),
        };

        self.advance();

        if self.peek_token()? == &Punctuator::Ampersand {
            self.advance();
            Ok(DataType::Pointer(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    fn parse_expr(&mut self) -> Result<Expression, String> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: i32) -> Result<Expression, String> {
        let mut left = self.parse_primary_expr()?;

        while let TokenKind::Punc(p) = self.peek_token()?
            && let Some(prec) = self.get_precedence(p)
        {
            let p = p.clone();
            if prec < min_prec {
                break;
            }

            self.advance();

            let right = self.parse_binary_expr(prec + 1).map_err(|e| {
                format!(
                    "{}. Context: parsing binary expression with operator '{}'{}",
                    e,
                    p,
                    self.get_context()
                )
            })?;
            left = Expression::BinaryOp(Box::new(left), p, Box::new(right));
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
            TokenKind::Literal(l) => {
                let lit = l.clone();
                self.advance();
                Ok(Expression::Literal(lit))
            }
            TokenKind::Punc(
                p @ (Punctuator::Plus
                | Punctuator::Minus
                | Punctuator::BitNot
                | Punctuator::Not
                | Punctuator::Inc
                | Punctuator::Dec),
            ) => {
                let p = p.clone();
                self.advance();
                let expr = self.parse_primary_expr()?;
                Ok(Expression::UnaryOp(p, Box::new(expr)))
            }
            TokenKind::Keyword(s @ Keyword::Sizeof) => {
                let s = s.to_string();
                self.advance();
                self.advance();

                let mut args = Vec::new();
                while self.peek_token()? != &Punctuator::RParen {
                    args.push(self.parse_expr()?);

                    if self.peek_token()? == &Punctuator::Comma {
                        self.advance();
                    }
                }
                self.expect(Punctuator::RParen)?;
                Ok(Expression::Call(Box::new(Expression::Identifier(s)), args))
            }
            TokenKind::Ident(s) => {
                let mut left = Expression::Identifier(s.to_owned());
                self.advance();

                loop {
                    match self.peek_token()? {
                        TokenKind::Punc(Punctuator::LParen) => {
                            self.advance();

                            let mut args = Vec::new();
                            while self.peek_token()? != &Punctuator::RParen && !self.is_eof() {
                                args.push(self.parse_expr()?);

                                if self.peek_token()? == &Punctuator::Comma {
                                    self.advance();
                                }
                            }
                            self.expect(Punctuator::RParen)?;
                            left = Expression::Call(Box::new(left), args);
                        }
                        TokenKind::Punc(Punctuator::LBracket) => {
                            self.advance();

                            let index = self.parse_expr()?;

                            self.expect(Punctuator::RBracket)?;
                            left = Expression::ArrayAccess(Box::new(left), Box::new(index));
                        }
                        TokenKind::Punc(Punctuator::Dot) => {
                            self.advance();

                            let field = self.parse_ident()?;
                            left = Expression::FieldAccess(Box::new(left), field);
                        }
                        _ => break,
                    };
                }
                Ok(left)
            }
            _ => Err(format!("invalid starting token {} for expression", token)),
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

    fn parse_literal(&mut self) -> Result<Expression, String> {
        let op = match self.peek_token()? {
            TokenKind::Punc(
                p @ (Punctuator::Plus
                | Punctuator::Minus
                | Punctuator::BitNot
                | Punctuator::Inc
                | Punctuator::Dec),
            ) => {
                let p = p.clone();
                self.advance();
                Some(p)
            }
            _ => None,
        };
        let literal = self.peek_token()?.clone();
        self.advance();
        match literal {
            TokenKind::Literal(l) => match op {
                Some(p) => Ok(Expression::UnaryOp(p, Box::new(Expression::Literal(l)))),
                None => Ok(Expression::Literal(l)),
            },
            _ => Err("wrong token kind".to_owned()),
        }
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
        let (t, _) = self
            .tokens
            .get(self.pos + after)
            .ok_or_else(|| format!("EoF at position {}", self.pos))?;
        // eprintln!("[DEBUG] peeking token: {:?}", s);
        Ok(t)
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

    fn expect_any<T>(&mut self, expected: T) -> Result<TokenKind, String>
    where
        T: IntoIterator,
        T::Item: Into<TokenKind>,
    {
        let token = self.peek_token()?;
        for e in expected {
            let t = e.into();
            if &t == token {
                self.advance();
                return Ok(t);
            }
        }
        Err(format!(
            "Expected '{}' but found {:?} '{}' at token position {} {}",
            token,
            self.peek_token()?,
            self.peek_token()?,
            self.pos,
            self.get_context()
        ))
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
