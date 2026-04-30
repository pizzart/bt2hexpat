use thiserror::Error;

use crate::ast_bt::{attr::*, data_type::*, stmt::*, template::*, token::*};
use std::collections::VecDeque;

#[derive(Debug, Error)]
pub enum ParseErrorType {
    #[error("reached end of token stream")]
    Eof,
    #[error("expected {0} ({0:?}), found {1}")]
    Expect(TokenKind, TokenKind),
    #[error("expected any of {0:?}, found {1}")]
    ExpectAny(Vec<TokenKind>, TokenKind),
    #[error("{0}")]
    Other(String),
}

type ParseResult<T> = Result<T, ParseErrorType>;

pub struct Parser {
    tokens: VecDeque<(TokenKind, String)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: VecDeque<(TokenKind, String)>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn get_context(&self) -> String {
        let mut context = "Context: ".to_string();
        for i in 0..=3 {
            if let Some(t) = self.tokens.get(self.pos + i).map(|(_, s)| s) {
                context.push_str(t);
                context.push(' ');
            }
        }
        context
    }

    pub fn parse(&mut self) -> ParseResult<BinaryTemplate> {
        let mut statements = Vec::new();

        while !self.is_eof() {
            statements.append(&mut self.parse_def_or_stmt()?);
        }

        Ok(BinaryTemplate {
            statements,
            metadata: TemplateMetadata::default(),
        })
    }

    fn parse_def_or_stmt(&mut self) -> ParseResult<Vec<Statement>> {
        match self.peek_token()? {
            TokenKind::Keyword(Keyword::Struct | Keyword::Union) => {
                if let Ok(d) = self.try_parse_var_def(false) {
                    Ok(d)
                } else {
                    let s = Statement::StructDef(self.parse_struct()?);
                    self.optional(Punctuator::Semicolon)?;
                    Ok(vec![s])
                }
            }
            TokenKind::Keyword(Keyword::Enum) => {
                if let Ok(d) = self.try_parse_var_def(false) {
                    Ok(d)
                } else {
                    Ok(vec![Statement::EnumDef(self.parse_enum()?)])
                }
            }
            TokenKind::Keyword(Keyword::Typedef) => self.parse_typedef().map(|e| vec![e]),
            TokenKind::Keyword(Keyword::Local) => self.parse_var_def(true),
            TokenKind::Keyword(Keyword::DataType(_) | Keyword::Unsigned | Keyword::Signed) => {
                self.parse_var_or_fn_def()
            }
            TokenKind::Ident(_) if matches!(self.peek_token_after(1)?, TokenKind::Ident(_)) => {
                self.parse_var_or_fn_def()
            }
            TokenKind::CPPDirective(s) => {
                let s = s.clone();
                self.advance()?;
                Ok(vec![Statement::CPPDirective(s)])
            }
            _ => self.parse_stmt().map(|e| vec![e]),
        }
    }

    fn parse_expr_stmt(&mut self) -> ParseResult<Statement> {
        let expr = self.parse_expr()?;

        // let s = self.peek_token()?;
        // if s.is_assign_op() {
        //     let sign = s.to_string();
        //     self.advance();
        //     let rhs = self.parse_expr()?;
        //     self.expect(Punctuator::Semicolon)?;
        //     return Ok(Statement::Assign {
        //         left: expr,
        //         sign,
        //         right: rhs,
        //     });
        // }

        self.expect(Punctuator::Semicolon)?;
        Ok(Statement::Expr(expr))
    }

    fn parse_typedef(&mut self) -> ParseResult<Statement> {
        self.expect(Keyword::Typedef)?;

        let base_ty = self.parse_type()?;

        eprintln!("[DEBUG] Parsing typedef type alias");

        let ident = self.parse_ident()?;
        eprintln!("[DEBUG] Typedef name: {}", ident);

        let mut ty = base_ty;
        while self.peek_token()? == &Punctuator::LBracket {
            self.advance()?;

            let size = if self.peek_token()? != &Punctuator::RBracket {
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };

            self.expect(Punctuator::RBracket)?;

            ty = DataType::Array(Box::new(ty), size);
        }

        let attrs = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.parse_attrs()?
        } else {
            Attributes(vec![])
        };

        self.expect(Punctuator::Semicolon)?;

        Ok(Statement::TypeDef { ident, ty, attrs })
    }

    fn parse_struct(&mut self) -> ParseResult<Struct> {
        let keyword = self.expect_any(vec![Keyword::Struct, Keyword::Union])?;
        let ty = match keyword {
            Keyword::Struct => StructType::Struct,
            Keyword::Union => StructType::Union,
            _ => panic!(),
        };

        let ident = if let TokenKind::Ident(_) = self.peek_token()? {
            Some(self.parse_ident()?)
        } else {
            None
        };
        eprintln!("[DEBUG] Parsing struct: {:?}", ident);

        let args = if self.peek_token()? == &Punctuator::LParen {
            let args = self.parse_args()?;
            self.expect(Punctuator::RParen)?;
            args
        } else {
            Args(vec![])
        };

        if self.peek_token()? == &Punctuator::Semicolon {
            self.advance()?;
            return Ok(Struct {
                ty,
                ident,
                args,
                body: Block(vec![]),
                attrs: Attributes(vec![]),
            });
        }

        eprintln!("[DEBUG] Entered struct body");
        let body = self.parse_braced_block()?;
        eprintln!("[DEBUG] Struct {:?} closed successfully", ident);

        let attrs = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.parse_attrs()?
        } else {
            Attributes(vec![])
        };

        Ok(Struct {
            ty,
            ident,
            args,
            body,
            attrs,
        })
    }

    fn parse_attrs(&mut self) -> ParseResult<Attributes> {
        self.expect(Punctuator::LAngledBracket)?;
        let mut attrs = vec![];
        while self.peek_token()? != &Punctuator::RAngledBracket {
            let ty = self
                .read_token()?
                .as_attribute()
                .map_err(|e| ParseErrorType::Other(e))?;
            self.expect(Punctuator::Assign)?;
            let value = self.parse_primary_expr()?;
            attrs.push(Attribute { ty, value });
            self.optional(Punctuator::Comma)?;
        }
        self.expect(Punctuator::RAngledBracket)?;
        Ok(Attributes(attrs))
    }

    fn attrs_get_pos(&self, attrs: &Attributes) -> Option<Expression> {
        attrs
            .iter()
            .find(|a| matches!(a.ty, AttributeType::Pos))
            .map(|a| a.value.clone())
    }

    fn parse_enum(&mut self) -> ParseResult<Enum> {
        self.expect(Keyword::Enum)?;

        let mut token = self.read_token()?;
        let ty = if token == Punctuator::LAngledBracket {
            token = self.read_token()?;
            Some(self.parse_type()?)
        } else {
            None
        };

        let ident = token.ident().ok().map(|s| s.to_owned());
        eprintln!("[DEBUG] Parsing enum: {:?}", ident);

        token = self.read_token()?;
        if token == Punctuator::Colon {
            self.parse_type()?;
        }

        self.expect(Punctuator::LBrace)?;

        let mut variants = Vec::new();

        while self.peek_token()? != &Punctuator::RBrace {
            let variant_name = self.parse_ident()?;
            eprintln!("[DEBUG] Enum variant: {}", variant_name);

            let value = if self.peek_token()? == &Punctuator::Assign {
                self.advance()?;
                Some(self.parse_literal()?)
            } else {
                None
            };

            variants.push((variant_name, value));

            self.optional(Punctuator::Comma)?;
        }

        self.expect(Punctuator::RBrace)?;

        let attrs = if self.peek_token()? == &Punctuator::LAngledBracket {
            self.parse_attrs()?
        } else {
            Attributes(vec![])
        };

        self.optional(Punctuator::Semicolon)?;

        Ok(Enum {
            ident,
            ty,
            variants,
            attrs,
        })
    }

    fn parse_ident(&mut self) -> ParseResult<String> {
        Ok(self
            .read_token()?
            .ident()
            .map_err(|s| ParseErrorType::Other(s))?
            .to_owned())
    }

    fn parse_var_or_fn_def(&mut self) -> ParseResult<Vec<Statement>> {
        let pos = self.pos;
        match self.parse_var_def(false) {
            Ok(d) => Ok(d),
            Err(e) => {
                dbg!(e);
                self.pos = pos;
                self.parse_fn_def().map(|s| vec![s])
            }
        }
    }

    fn parse_args(&mut self) -> ParseResult<Args> {
        let mut args = vec![];
        while self.peek_token()? != &Punctuator::RParen {
            let data_type = self.parse_type()?;
            let ident = self.parse_ident()?;
            args.push((data_type, ident));
            self.optional(Punctuator::Comma)?;
        }
        Ok(Args(args))
    }

    fn parse_fn_def(&mut self) -> ParseResult<Statement> {
        let ty = self.parse_type()?;

        let ident = self.parse_ident()?;
        self.expect(Punctuator::LParen)?;
        let args = self.parse_args()?;
        self.expect(Punctuator::RParen)?;
        let body = self.parse_any_block()?;
        self.optional(Punctuator::Semicolon)?;
        Ok(Statement::FnDef {
            ty,
            ident,
            args,
            body,
        })
    }

    fn parse_var_def(&mut self, local: bool) -> ParseResult<Vec<Statement>> {
        let mut stmts: Vec<Statement> = vec![];
        if local {
            self.expect(Keyword::Local)?;
        }

        let base_ty = self.parse_type()?;

        loop {
            let ident = self.parse_ident()?;

            let mut ty = base_ty.clone();
            if self.peek_token()? == &Punctuator::LParen {
                self.advance()?;

                let mut args = vec![];
                while self.peek_token()? != &Punctuator::RParen {
                    let expr = self.parse_expr()?;
                    args.push(expr);
                    self.optional(Punctuator::Comma)?;
                }

                self.expect(Punctuator::RParen)?;

                ty = DataType::Args(Box::new(ty), args);
            }
            if self.peek_token()? == &Punctuator::LBracket {
                self.advance()?;

                let size = if self.peek_token()? != &Punctuator::RBracket {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };

                self.expect(Punctuator::RBracket)?;

                ty = DataType::Array(Box::new(ty), size);
            }

            let value = if self.peek_token()? == &Punctuator::Assign {
                self.advance()?;

                Some(self.parse_expr()?)
            } else {
                None
            };

            let bits = if self.peek_token()? == &Punctuator::Colon {
                self.advance()?;

                match self.parse_literal()? {
                    Expression::Literal(l) => l.int(),
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

    fn try_parse_var_def(&mut self, local: bool) -> ParseResult<Vec<Statement>> {
        let pos = self.pos;
        match self.parse_var_def(local) {
            Ok(d) => Ok(d),
            Err(e) => {
                self.pos = pos;
                Err(e)
            }
        }
    }

    fn parse_if(&mut self) -> ParseResult<Statement> {
        self.expect(Keyword::If)?;
        self.expect(Punctuator::LParen)?;

        eprintln!("[DEBUG] Parsing if condition");
        let condition = self.parse_expr()?;

        self.expect(Punctuator::RParen)?;

        let then_block = self.parse_any_block()?;
        let else_block = if self.peek_token()? == &Keyword::Else {
            self.advance()?;
            Some(self.parse_any_block()?)
        } else {
            None
        };

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while(&mut self) -> ParseResult<Statement> {
        self.expect(Keyword::While)?;

        self.expect(Punctuator::LParen)?;

        eprintln!("[DEBUG] Parsing while condition");
        let condition = self.parse_expr()?;

        self.expect(Punctuator::RParen)?;

        let body = self.parse_any_block()?;

        Ok(Statement::While { condition, body })
    }

    fn parse_for(&mut self) -> ParseResult<Statement> {
        self.expect(Keyword::For)?;

        self.expect(Punctuator::LParen)?;

        eprintln!("[DEBUG] Parsing for condition");
        let init = self.parse_expr()?;
        self.expect(Punctuator::Semicolon)?;
        let test = self.parse_expr()?;
        self.expect(Punctuator::Semicolon)?;
        let upd = self.parse_expr()?;
        self.optional(Punctuator::Semicolon)?;

        self.expect(Punctuator::RParen)?;

        let body = self.parse_any_block()?;

        Ok(Statement::For {
            init,
            test,
            upd,
            body,
        })
    }

    fn parse_any_block(&mut self) -> ParseResult<Block> {
        if self.peek_token()? == &Punctuator::LBrace {
            self.parse_braced_block()
        } else {
            self.parse_inline_block()
        }
    }

    fn parse_braced_block(&mut self) -> ParseResult<Block> {
        let mut stmts = vec![];
        self.expect(Punctuator::LBrace)?;
        while self.peek_token()? != &Punctuator::RBrace {
            stmts.append(&mut self.parse_def_or_stmt()?);
        }
        self.expect(Punctuator::RBrace)?;
        Ok(Block(stmts))
    }

    fn parse_inline_block(&mut self) -> ParseResult<Block> {
        Ok(Block(self.parse_def_or_stmt()?))
    }

    fn parse_switch(&mut self) -> ParseResult<Statement> {
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
            let token = self.read_token()?;

            match token {
                TokenKind::Keyword(Keyword::Case) => {
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
                    self.expect(Punctuator::Colon)?;
                    let mut stmts = vec![];
                    while self.peek_token()? != &Punctuator::RBrace {
                        stmts.append(&mut self.parse_def_or_stmt()?);
                    }
                    default.replace(Block(stmts));
                }
                _ => {
                    return Err(ParseErrorType::Other(format!(
                        "invalid token encountered in switch statement, expected case or default, got {}",
                        token
                    )));
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

    fn parse_return(&mut self) -> ParseResult<Statement> {
        eprintln!("[DEBUG] Parsing return statement");
        self.expect(Keyword::Return)?;

        let value = if self.peek_token()? != &Punctuator::Semicolon {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(Punctuator::Semicolon)?;
        Ok(Statement::Return(value))
    }

    fn parse_stmt(&mut self) -> ParseResult<Statement> {
        let token = self.peek_token()?;
        eprintln!("[DEBUG] parse_statement: token = '{}'", token);

        match token {
            TokenKind::Keyword(Keyword::If) => self.parse_if(),
            TokenKind::Keyword(Keyword::While) => self.parse_while(),
            TokenKind::Keyword(Keyword::For) => self.parse_for(),
            TokenKind::Keyword(Keyword::Switch) => self.parse_switch(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Keyword(Keyword::Break) => {
                self.expect(Keyword::Break)?;
                self.expect(Punctuator::Semicolon)?;
                Ok(Statement::Break)
            }
            TokenKind::Keyword(Keyword::Continue) => {
                self.expect(Keyword::Continue)?;
                self.expect(Punctuator::Semicolon)?;
                Ok(Statement::Continue)
            }
            TokenKind::Punc(Punctuator::LBrace) => {
                let block = self.parse_braced_block()?;
                Ok(Statement::Block(block))
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_type(&mut self) -> ParseResult<DataType> {
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

    fn parse_basic_type(&mut self) -> ParseResult<DataType> {
        let token = self.read_token()?;

        let base_type = match token {
            TokenKind::Keyword(Keyword::Unsigned | Keyword::Signed) => {
                let next_token = self.read_token()?;
                match next_token {
                    TokenKind::Keyword(Keyword::DataType(dt)) => {
                        if token == Keyword::Unsigned {
                            dt.to_unsigned()
                        } else {
                            dt.to_signed()
                        }
                    }
                    _ => {
                        return Err(ParseErrorType::Other(
                            "no type after 'unsigned' found".to_owned(),
                        ));
                    }
                }
            }
            TokenKind::Keyword(Keyword::DataType(dt)) => dt,
            TokenKind::Ident(i) => DataType::Ident(i),
            t => return Err(ParseErrorType::Other(format!("Nonsense datatype {}", t))),
        };

        if self.peek_token()? == &Punctuator::Ampersand {
            self.advance()?;
            Ok(DataType::Pointer(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    fn parse_expr(&mut self) -> ParseResult<Expression> {
        self.parse_binary_expr(0)
    }

    fn parse_binary_expr(&mut self, min_prec: i32) -> ParseResult<Expression> {
        let mut left = self.parse_primary_expr()?;

        while let TokenKind::Punc(p) = self.peek_token()?
            && let Some(prec) = self.get_precedence(p)
        {
            let p = p.clone();
            if prec < min_prec {
                break;
            }
            self.advance()?;

            let right = self.parse_binary_expr(prec + 1)?;
            left = Expression::BinaryOp(Box::new(left), p, Box::new(right));
        }

        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> ParseResult<Expression> {
        let token = self.read_token()?;
        match token {
            TokenKind::Punc(Punctuator::LParen) => {
                let saved_pos = self.pos;
                let is_cast = self.is_cast();

                if is_cast {
                    self.pos = saved_pos;
                    let cast_type = Box::new(self.parse_type()?);

                    self.expect(Punctuator::RParen)?;

                    // Parse the expression being cast
                    let expr = Box::new(self.parse_primary_expr()?);
                    eprintln!("[DEBUG] Parsed cast to type {:?}", cast_type);

                    Ok(Expression::Cast(cast_type, expr))
                } else {
                    // Regular parenthesized expression
                    let expr = self.parse_expr()?;

                    self.expect(Punctuator::RParen)?;
                    Ok(expr)
                }
            }
            TokenKind::Literal(l) => Ok(Expression::Literal(l)),
            TokenKind::Punc(
                p @ (Punctuator::Plus
                | Punctuator::Minus
                | Punctuator::BitNot
                | Punctuator::Not
                | Punctuator::Inc
                | Punctuator::Dec),
            ) => {
                let expr = self.parse_primary_expr()?;
                Ok(Expression::UnaryOp(
                    p,
                    Box::new(expr),
                    UnaryPosition::Prefix,
                ))
            }
            TokenKind::Keyword(s @ Keyword::Sizeof) => {
                let s = s.to_string();
                self.expect(Punctuator::LParen)?;
                let arg = self.parse_expr()?;
                self.expect(Punctuator::RParen)?;
                Ok(Expression::Call(
                    Box::new(Expression::Identifier(s)),
                    vec![arg],
                ))
            }
            TokenKind::Keyword(Keyword::Color(c)) => {
                let s = c.to_string();
                Ok(Expression::Identifier(s))
            }
            TokenKind::Ident(s) => {
                let mut left = Expression::Identifier(s);

                loop {
                    match self.peek_token()? {
                        TokenKind::Punc(Punctuator::LParen) => {
                            self.advance()?;

                            let mut args = Vec::new();
                            while self.peek_token()? != &Punctuator::RParen {
                                args.push(self.parse_expr()?);
                                self.optional(Punctuator::Comma)?;
                            }
                            self.expect(Punctuator::RParen)?;
                            left = Expression::Call(Box::new(left), args);
                        }
                        TokenKind::Punc(Punctuator::LBracket) => {
                            self.advance()?;

                            let index = self.parse_expr()?;

                            self.expect(Punctuator::RBracket)?;
                            left = Expression::ArrayAccess(Box::new(left), Box::new(index));
                        }
                        TokenKind::Punc(Punctuator::Dot) => {
                            self.advance()?;

                            let field = self.parse_ident()?;
                            left = Expression::FieldAccess(Box::new(left), field);
                        }
                        TokenKind::Punc(p @ (Punctuator::Inc | Punctuator::Dec)) => {
                            let p = p.clone();
                            self.advance()?;

                            left = Expression::UnaryOp(p, Box::new(left), UnaryPosition::Postfix);
                        }
                        _ => break,
                    }
                }
                Ok(left)
            }
            _ => Err(ParseErrorType::Other(format!(
                "invalid starting token {} for expression",
                token
            ))),
        }
    }

    fn is_cast(&mut self) -> bool {
        let token = if let Ok(t) = self.peek_token() {
            t
        } else {
            return false;
        };

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

    fn parse_literal(&mut self) -> ParseResult<Expression> {
        let op = match self.peek_token()? {
            TokenKind::Punc(
                p @ (Punctuator::Plus
                | Punctuator::Minus
                | Punctuator::BitNot
                | Punctuator::Inc
                | Punctuator::Dec),
            ) => {
                let p = p.clone();
                self.advance()?;
                Some(p)
            }
            _ => None,
        };
        let literal = self.read_token()?.clone();
        match literal {
            TokenKind::Literal(l) => match op {
                Some(p) => Ok(Expression::UnaryOp(
                    p,
                    Box::new(Expression::Literal(l)),
                    UnaryPosition::Prefix,
                )),
                None => Ok(Expression::Literal(l)),
            },
            _ => Err(ParseErrorType::Other("wrong token kind".to_owned())),
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
            Punctuator::BitLeftShift | Punctuator::BitRightShift => Some(8),
            Punctuator::Plus | Punctuator::Minus => Some(9),
            Punctuator::Asterisk | Punctuator::Div | Punctuator::Mod => Some(10),
            Punctuator::Assign
            | Punctuator::DivAssign
            | Punctuator::MinusAssign
            | Punctuator::ModAssign
            | Punctuator::MultAssign
            | Punctuator::PlusAssign
            | Punctuator::BitAndAssign
            | Punctuator::BitOrAssign
            | Punctuator::BitRightShiftAssign
            | Punctuator::BitLeftShiftAssign
            | Punctuator::BitXorAssign => Some(0),
            _ => None,
        }
    }

    fn peek_token(&self) -> ParseResult<&TokenKind> {
        self.peek_token_after(0)
    }

    fn peek_token_after(&self, after: usize) -> ParseResult<&TokenKind> {
        let (t, _) = self
            .tokens
            .get(self.pos + after)
            .ok_or(ParseErrorType::Eof)?;
        Ok(t)
    }

    fn read_token(&mut self) -> ParseResult<TokenKind> {
        let t = self
            .tokens
            .get(self.pos)
            .cloned()
            .ok_or(ParseErrorType::Eof)
            .map(|(t, _)| t);
        self.pos += 1;
        t
    }

    fn advance(&mut self) -> ParseResult<()> {
        if self.is_eof() {
            return Err(ParseErrorType::Eof);
        }
        self.pos += 1;
        Ok(())
    }

    fn expect<T>(&mut self, expected: T) -> ParseResult<()>
    where
        T: Into<TokenKind>,
    {
        let token: TokenKind = expected.into();
        let next = self.read_token()?;

        if next == token {
            Ok(())
        } else {
            Err(ParseErrorType::Expect(token, next))
        }
    }

    fn optional<T>(&mut self, expected: T) -> ParseResult<()>
    where
        T: Into<TokenKind>,
    {
        let token: TokenKind = expected.into();

        if self.peek_token()? == &token {
            self.read_token()?;
        }
        Ok(())
    }

    fn expect_any<T>(&mut self, expected: T) -> ParseResult<T::Item>
    where
        T: IntoIterator + std::fmt::Debug + Clone,
        T::Item: Into<TokenKind> + Clone,
    {
        let err = ParseErrorType::ExpectAny(
            expected.clone().into_iter().map(|t| t.into()).collect(),
            self.peek_token()?.clone(),
        );
        let token = self.peek_token()?;
        for e in expected {
            let t = e.clone().into();
            if &t == token {
                self.advance()?;
                return Ok(e);
            }
        }
        Err(err)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}
