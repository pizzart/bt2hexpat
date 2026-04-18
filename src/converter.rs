use crate::ast::*;

pub struct HexPatConverter {
    indent: usize,
}

impl HexPatConverter {
    pub fn new() -> Self {
        HexPatConverter { indent: 0 }
    }

    pub fn convert(&mut self, template: &Template) -> Result<String, String> {
        let mut stmts = template.statements.clone();
        self.reorder_stmts(&mut stmts);

        let mut output = String::new();

        // Add pragma header
        output.push_str("#pragma description Converted from 010 Editor Binary Template\n");
        if let Some(desc) = &template.metadata.description {
            output.push_str(&format!("#pragma description {}\n", desc));
        }
        // output.push_str("\nimport std.mem;\n\n");

        // Convert statements
        for statement in &stmts {
            output.push_str(&self.write_statement(statement)?);
        }

        Ok(output)
    }

    fn reorder_in_datatype(&self, datatype: &mut DataType) -> Vec<Statement> {
        let mut to_move = vec![];
        match datatype {
            DataType::Struct(s) => {
                let new =
                    DataType::Custom(s.ident.to_owned().unwrap_or_else(|| "NONAME".to_string()));
                let s = std::mem::replace(datatype, new);
                to_move.push(Statement::StructDef(match s {
                    DataType::Struct(s) => s,
                    _ => panic!("somehow, by pure chance, the value is not the same type anymore"),
                }));
            }
            DataType::Enum(e) => {
                let new =
                    DataType::Custom(e.ident.to_owned().unwrap_or_else(|| "NONAME".to_string()));
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
                // for (i, s) in st.0.into_iter().enumerate() {
                //     if matches!(s, Statement::StructDef(_) | Statement::EnumDef(_)) {
                //         let s = st.0.remove(i);
                //         stmts.push(s);
                //     }
                // }
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
                for item in s.body.iter_mut() {
                    match item {
                        StructItem::Field(StructField { ident: _, ty }) => {
                            to_move.append(&mut self.reorder_in_datatype(ty));
                        }
                        StructItem::Statement(s) => {
                            to_move.append(&mut self.reorder_stmt(s));
                        }
                    }
                }
            }
            Statement::Switch { expr: _, cases } => {
                for case in cases {
                    to_move.append(&mut self.reorder_stmts(&mut case.0));
                }
            }
            Statement::TypeDef { ident, ty } => match ty {
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
            Statement::VarDecl {
                ident: _,
                ty,
                value: _,
                local: _,
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
            if let Statement::TypeDef { ident: _, ty } = stmt
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

    fn write_statement(&mut self, stmt: &Statement) -> Result<String, String> {
        match stmt {
            Statement::StructDef(s) => self.write_struct(s),
            Statement::Expr(expr) => Ok(format!("{}{};\n", self.indent(), self.write_expr(expr)?)),
            Statement::TypeDef { ident, ty } => self.write_typedef(ident, ty),
            Statement::EnumDef(e) => self.write_enum(e),
            Statement::VarDecl {
                ident,
                ty,
                value,
                local: _,
            } => self.write_var_decl(ident, ty, value),
            Statement::Assign { left, right } => {
                let l = self.write_expr(left)?;
                let r = self.write_expr(right)?;
                Ok(format!("{} = {}", l, r))
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => self.write_if(condition, then_block, else_block),
            Statement::While { condition, body } => self.write_while(condition, &body.0),
            Statement::Block(block) => {
                let mut output = String::from("{\n");
                for stmt in block.0.iter() {
                    output.push_str(&self.write_statement(stmt)?);
                }
                output.push_str("}\n");
                Ok(output)
            }
            Statement::Return(expr) => Ok(expr.as_ref().map_or_else(
                || String::new(),
                |e| {
                    format!(
                        "{}return {};\n",
                        self.indent(),
                        self.write_expr(&e).unwrap()
                    )
                },
            )),
            Statement::Switch { expr, cases } => {
                let mut output =
                    format!("{}match ({}) {{\n", self.indent(), self.write_expr(expr)?);
                self.add_indent();
                for (case_idx, case_body) in cases.iter().enumerate() {
                    if case_idx == cases.len() - 1 && !cases.is_empty() {
                        output.push_str(&format!("{}(_): {{\n", self.indent()));
                    } else {
                        output.push_str(&format!("{}({}): {{\n", self.indent(), case_idx));
                    }
                    self.add_indent();
                    for stmt in case_body.0.iter() {
                        if matches!(stmt, Statement::Expr(Expression::Identifier(i)) if i == "break")
                        {
                            continue;
                        }
                        output.push_str(&self.write_statement(&stmt)?);
                    }
                    output.push_str(&format!("{}}}\n", self.indent()));
                    self.rem_indent();
                    // output.push_str(&format!("{}        break;\n", self.indent_str()));
                }
                self.rem_indent();
                output.push_str(&format!("{}}}\n", self.indent()));
                Ok(output)
            }
        }
    }

    fn write_struct(&mut self, s: &Struct) -> Result<String, String> {
        let mut output = format!(
            "{}struct {} {{\n",
            self.indent(),
            s.ident.clone().unwrap_or_else(|| "NONAME".to_string())
        );
        self.add_indent();
        for content in &s.body {
            output.push_str(&self.write_struct_item(content)?);
        }
        self.rem_indent();
        output.push_str(&format!("{}}}\n", self.indent()));
        Ok(output)
    }

    fn write_var_decl(
        &mut self,
        ident: &str,
        ty: &DataType,
        value: &Option<Expression>,
    ) -> Result<String, String> {
        let type_str = self.write_type(ty, false)?;
        let mut output = format!("{}{} {}", self.indent(), type_str, ident);

        if let DataType::Array(_, size) = ty {
            if let Some(size_expr) = size {
                output.push_str(&format!("[{}]", self.write_expr(size_expr)?));
            } else {
                output.push_str("[]");
            }
        }

        if let Some(expr) = value {
            output.push_str(&format!(" = {}", self.write_expr(expr)?));
        }

        output.push_str(";\n");
        Ok(output)
    }

    fn write_struct_item(&mut self, content: &StructItem) -> Result<String, String> {
        match content {
            StructItem::Field(field) => self.write_var_decl(&field.ident, &field.ty, &None),
            StructItem::Statement(stmt) => self.write_statement(stmt),
        }
    }

    fn write_typedef(&mut self, name: &str, ty: &DataType) -> Result<String, String> {
        Ok(format!(
            "using {} = {};\n\n",
            name,
            self.write_type(ty, true)?
        ))
    }

    fn write_enum(&mut self, e: &Enum) -> Result<String, String> {
        let mut output = format!(
            "enum {} : {} {{\n",
            e.ident.clone().unwrap_or_else(|| String::new()),
            e.ty.as_ref()
                .map_or_else(|| "u32".to_string(), |t| t.to_string())
        );

        self.add_indent();
        for (var_name, value) in &e.variants {
            output.push_str(&format!("{}  {}", self.indent(), var_name));
            if let Some(v) = value {
                output.push_str(&format!(" = {}", v));
            }
            output.push_str(",\n");
        }
        self.rem_indent();

        output.push_str("};\n\n");
        Ok(output)
    }

    fn write_if(
        &mut self,
        condition: &Expression,
        then_block: &Block,
        else_block: &Option<Block>,
    ) -> Result<String, String> {
        let mut output = format!("{}if ({}) {{\n", self.indent(), self.write_expr(condition)?);

        self.add_indent();
        for stmt in then_block.0.iter() {
            output.push_str(&self.write_statement(stmt)?);
        }
        self.rem_indent();

        output.push_str(&format!("{}}}", self.indent()));

        if let Some(else_stmts) = else_block {
            let else_if = matches!(else_stmts.0.get(0), Some(Statement::If { .. }));

            // match else_stmts.get(0) {
            //     Some(Statement::If { .. }) => output.push_str(" else "),
            //     _ => output.push_str(" else {\n"),
            // }
            output.push_str(if !else_if { " else {\n" } else { " else" });
            for stmt in else_stmts.0.iter() {
                output.push_str(&self.write_statement(stmt)?);
            }
            if !else_if {
                output.push_str(&format!("{}}}", self.indent()));
            }
        }

        output.push('\n');
        Ok(output)
    }

    fn write_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<String, String> {
        let mut output = format!(
            "{}while ({}) {{\n",
            self.indent(),
            self.write_expr(condition)?
        );

        self.add_indent();
        for stmt in body {
            output.push_str(&self.write_statement(stmt)?);
        }
        self.rem_indent();

        output.push_str(&format!("{}}}\n", self.indent()));
        Ok(output)
    }

    fn write_type(&mut self, ty: &DataType, array_braces: bool) -> Result<String, String> {
        match ty {
            DataType::I8 => Ok("char".to_string()),
            DataType::U8 => Ok("u8".to_string()),
            DataType::I16 => Ok("s16".to_string()),
            DataType::U16 => Ok("u16".to_string()),
            DataType::I32 => Ok("s32".to_string()),
            DataType::U32 => Ok("u32".to_string()),
            DataType::I64 => Ok("s64".to_string()),
            DataType::U64 => Ok("u64".to_string()),
            DataType::HFloat => Ok("HFLOAT".to_string()),
            DataType::Float => Ok("float".to_string()),
            DataType::Double => Ok("double".to_string()),
            DataType::Custom(name) => Ok(name.clone()),
            DataType::Struct(s) => self.write_struct(s),
            DataType::Enum(e) => self.write_enum(e),
            DataType::Array(base_ty, e) => {
                if array_braces {
                    Ok(format!(
                        "{}[{}]",
                        self.write_type(base_ty, true)?,
                        e.as_ref()
                            .map_or_else(|| String::new(), |exp| self.write_expr(&*exp).unwrap())
                    ))
                } else {
                    self.write_type(base_ty, false)
                }
            }
            DataType::Unused => Ok("UNUSED".to_string()),
        }
    }

    fn write_expr(&mut self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Literal(lit) => Ok(lit.clone()),
            Expression::Identifier(var) => Ok(var.clone()),
            Expression::UnaryOp(op, right) => Ok(format!("{}{}", op, self.write_expr(right)?)),
            Expression::BinaryOp(left, op, right) => Ok(format!(
                "{} {} {}",
                self.write_expr(left)?,
                op,
                self.write_expr(right)?
            )),
            Expression::Call(name, args) => {
                let args_str = args
                    .iter()
                    .map(|a| self.write_expr(a))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", name, args_str))
            }
            Expression::Cast(ty, expr) => Ok(format!(
                "({}) {}",
                self.write_type(ty, false)?,
                self.write_expr(expr)?
            )),
            Expression::FieldAccess(expr, field) => {
                Ok(format!("{}.{}", self.write_expr(expr)?, field))
            }
            Expression::ArrayAccess(expr, index) => Ok(format!(
                "{}[{}]",
                self.write_expr(expr)?,
                self.write_expr(index)?
            )),
        }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent)
    }

    fn add_indent(&mut self) {
        self.indent += 2;
    }

    fn rem_indent(&mut self) {
        self.indent -= 2;
    }
}
