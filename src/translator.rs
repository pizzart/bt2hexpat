use crate::{
    ast::{data_type::*, stmt::*, template::*},
    traits::to_imhex::{ToImhex, ToImhexErr},
};

pub struct Translator;

impl Translator {
    pub fn new() -> Self {
        Translator
    }

    pub fn translate(&mut self, template: &mut Template) -> Result<String, ToImhexErr> {
        self.reorder_stmts(&mut template.statements);
        template.try_to_imhex()
    }

    fn reorder_in_datatype(&self, datatype: &mut DataType) -> Vec<Statement> {
        let mut to_move = vec![];
        match datatype {
            DataType::Struct(s) => {
                let new =
                    DataType::Ident(s.ident.to_owned().unwrap_or_else(|| "NONAME".to_string()));
                let s = std::mem::replace(datatype, new);
                to_move.push(Statement::StructDef(match s {
                    DataType::Struct(s) => s,
                    _ => panic!("somehow, by pure chance, the value is not the same type anymore"),
                }));
            }
            DataType::Enum(e) => {
                let new =
                    DataType::Ident(e.ident.to_owned().unwrap_or_else(|| "NONAME".to_string()));
                let e = std::mem::replace(datatype, new);
                to_move.push(Statement::EnumDef(match e {
                    DataType::Enum(e) => *e,
                    _ => panic!("somehow, by pure chance, the value is not the same type anymore"),
                }));
            }
            DataType::Array(dt, _) => {
                to_move.append(&mut self.reorder_in_datatype(dt));
            }
            _ => (),
        }
        to_move
    }

    fn reorder_stmt(&self, stmt: &mut Statement) -> Vec<Statement> {
        let mut to_move = vec![];
        match stmt {
            Statement::Block(st) => {
                to_move.append(&mut self.reorder_stmts(&mut st.0));
            }
            Statement::EnumDef(e) => {
                if let Some(dt) = &mut e.ty {
                    to_move.append(&mut self.reorder_in_datatype(dt));
                }
            }
            Statement::If {
                condition: _,
                then_block,
                else_block,
            } => {
                to_move.append(&mut self.reorder_stmts(&mut then_block.0));
                if let Some(e) = else_block {
                    to_move.append(&mut self.reorder_stmts(&mut e.0));
                }
            }
            Statement::StructDef(s) => {
                to_move.append(&mut self.reorder_stmts(&mut s.body.0));
            }
            Statement::Switch {
                expr: _,
                cases,
                default,
            } => {
                for (_, body) in cases {
                    to_move.append(&mut self.reorder_stmts(&mut body.0));
                }
                if let Some(body) = default {
                    to_move.append(&mut self.reorder_stmts(&mut body.0));
                }
            }
            Statement::TypeDef {
                ident,
                ty,
                attrs: _,
            } => match ty {
                DataType::Struct(_) => {
                    let new = DataType::Unused;
                    let s = std::mem::replace(ty, new);
                    to_move.push(Statement::StructDef(match s {
                        DataType::Struct(mut s) => {
                            s.ident.replace(ident.to_owned());
                            s
                        }
                        _ => panic!(
                            "somehow, by pure chance, the value is not the same type anymore"
                        ),
                    }));
                }
                DataType::Enum(_) => {
                    let new = DataType::Unused;
                    let e = std::mem::replace(ty, new);
                    to_move.push(Statement::EnumDef(match e {
                        DataType::Enum(mut e) => {
                            e.ident = Some(ident.to_owned());
                            *e
                        }
                        _ => panic!(
                            "somehow, by pure chance, the value is not the same type anymore"
                        ),
                    }));
                }
                DataType::Array(dt, _) => {
                    to_move.append(&mut self.reorder_in_datatype(dt));
                }
                _ => (),
            },
            Statement::VarDef {
                ident: _,
                ty,
                value: _,
                local: _,
                bits: _,
                attrs: _,
                pos: _,
            } => {
                to_move.append(&mut self.reorder_in_datatype(ty));
            }
            _ => (),
        }
        to_move
    }

    fn reorder_stmts(&self, stmts: &mut Vec<Statement>) -> Vec<Statement> {
        let mut to_add = vec![];
        let mut to_destroy = vec![];
        for (i, stmt) in stmts.iter_mut().enumerate() {
            let new = self.reorder_stmt(stmt);
            if !new.is_empty() {
                to_add.push((i, new))
            }
        }
        for (i, stmt) in stmts.iter().enumerate() {
            if let Statement::TypeDef {
                ident: _,
                ty,
                attrs: _,
            } = stmt
                && ty == &DataType::Unused
            {
                to_destroy.push(i);
            }
        }
        for i in to_destroy.into_iter().rev() {
            stmts.remove(i);
        }
        for (i, stmts_to_add) in to_add.clone() {
            for stmt in stmts_to_add.into_iter().rev() {
                stmts.insert(i, stmt);
            }
        }
        return to_add.into_iter().flat_map(|(_, stmt)| stmt).collect();
    }
}
