use crate::ast::*;

pub struct HexPatConverter {
    indent: usize,
}

impl HexPatConverter {
    pub fn new() -> Self {
        HexPatConverter { indent: 0 }
    }

    pub fn convert(&mut self, template: &Template) -> Result<String, String> {
        let mut output = String::new();

        // Add pragma header
        output.push_str("#pragma description Converted from 010 Editor Binary Template\n");
        if let Some(desc) = &template.metadata.description {
            output.push_str(&format!("#pragma description {}\n", desc));
        }
        output.push_str("\nimport std.mem;\n\n");

        // Convert statements
        for statement in &template.statements {
            output.push_str(&self.convert_statement(statement)?);
        }

        Ok(output)
    }

    fn convert_statement(&mut self, stmt: &Statement) -> Result<String, String> {
        match stmt {
            Statement::StructDef { ident, body } => self.convert_struct(ident, body),
            Statement::Expr(expr) => Ok(format!(
                "{}{};\n",
                self.indent(),
                self.convert_expression(expr)?
            )),
            Statement::TypeDef { ident, ty } => self.convert_typedef(ident, ty),
            Statement::EnumDef { ident, variants } => self.convert_enum(ident, variants),
            Statement::VarDecl {
                ident,
                ty,
                value,
                local: _,
            } => self.convert_var_decl(ident, ty, value),
            Statement::Assign { left, right } => {
                let l = self.convert_expression(left)?;
                let r = self.convert_expression(right)?;
                Ok(format!("{} = {}", l, r))
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => self.convert_if(condition, then_block, else_block),
            Statement::While { condition, body } => self.convert_while(condition, &body.0),
            Statement::FunctionCall(name, args) => {
                Ok(format!(
                    "{}{}",
                    name,
                    args.iter()
                        .filter_map(|a| self.convert_expression(a).ok())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                // Ok(format!("{}// Function call: {}\n", self.indent_str(), name))
            }
            Statement::Block(block) => {
                let mut output = String::from("{\n");
                for stmt in block.0.iter() {
                    output.push_str(&self.convert_statement(stmt)?);
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
                        self.convert_expression(&e).unwrap()
                    )
                },
            )),
            Statement::Switch { expr, cases } => {
                let mut output = format!(
                    "{}match ({}) {{\n",
                    self.indent(),
                    self.convert_expression(expr)?
                );
                self.add_indent();
                for (case_idx, case_body) in cases.iter().enumerate() {
                    if case_idx == cases.len() - 1 && !cases.is_empty() {
                        output.push_str(&format!("{}(_):\n", self.indent()));
                    } else {
                        output.push_str(&format!("{}({}):\n", self.indent(), case_idx));
                    }
                    self.add_indent();
                    for stmt in case_body.0.iter() {
                        if matches!(stmt, Statement::Expr(Expression::Identifier(i)) if i == "break")
                        {
                            continue;
                        }
                        output.push_str(&self.convert_statement(&stmt)?);
                    }
                    self.rem_indent();
                    // output.push_str(&format!("{}        break;\n", self.indent_str()));
                }
                self.rem_indent();
                output.push_str(&format!("{}}}\n", self.indent()));
                Ok(output)
            }
        }
    }

    fn convert_struct(&mut self, ident: &str, body: &Vec<StructItem>) -> Result<String, String> {
        let mut output = format!("{}struct {} {{\n", self.indent(), ident);
        self.add_indent();
        for content in body {
            output.push_str(&self.convert_struct_item(content)?);
        }
        self.rem_indent();
        output.push_str(&format!("{}}}\n", self.indent()));
        Ok(output)
    }

    fn convert_var_decl(
        &mut self,
        ident: &str,
        ty: &DataType,
        value: &Option<Expression>,
    ) -> Result<String, String> {
        let type_str = self.convert_type(ty)?;
        let mut output = format!("{}{} {}", self.indent(), type_str, ident);

        if let DataType::Array(_, size) = ty {
            if let Some(size_expr) = size {
                output.push_str(&format!("[{}]", self.convert_expression(size_expr)?));
            } else {
                output.push_str("[]");
            }
        }

        if let Some(expr) = value {
            output.push_str(&format!(" = {}", self.convert_expression(expr)?));
        }

        output.push_str(";\n");
        Ok(output)
    }

    fn convert_struct_item(&mut self, content: &StructItem) -> Result<String, String> {
        match content {
            StructItem::Field(field) => self.convert_var_decl(&field.ident, &field.ty, &None),
            StructItem::Statement(stmt) => self.convert_statement(stmt),
        }
    }

    fn convert_typedef(&mut self, name: &str, ty: &DataType) -> Result<String, String> {
        Ok(format!("using {} = {};\n\n", name, self.convert_type(ty)?))
    }

    fn convert_enum(
        &mut self,
        name: &str,
        variants: &[(String, Option<i64>)],
    ) -> Result<String, String> {
        let mut output = format!("enum {} : u32 {{\n", name);

        self.add_indent();
        for (var_name, value) in variants {
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

    fn convert_if(
        &mut self,
        condition: &Expression,
        then_block: &Block,
        else_block: &Option<Block>,
    ) -> Result<String, String> {
        let mut output = format!(
            "{}if ({}) {{\n",
            self.indent(),
            self.convert_expression(condition)?
        );

        self.add_indent();
        for stmt in then_block.0.iter() {
            output.push_str(&self.convert_statement(stmt)?);
        }
        self.rem_indent();

        output.push_str(&format!("{}}}", self.indent()));

        if let Some(else_stmts) = else_block {
            let else_if = matches!(else_stmts.0.get(0), Some(Statement::If { .. }));

            // match else_stmts.get(0) {
            //     Some(Statement::If { .. }) => output.push_str(" else "),
            //     _ => output.push_str(" else {\n"),
            // }
            output.push_str(if !else_if { " else {\n" } else { " else " });
            for stmt in else_stmts.0.iter() {
                output.push_str(&self.convert_statement(stmt)?);
            }
            if !else_if {
                output.push_str(&format!("{}}}", self.indent()));
            }
        }

        output.push('\n');
        Ok(output)
    }

    fn convert_while(
        &mut self,
        condition: &Expression,
        body: &[Statement],
    ) -> Result<String, String> {
        let mut output = format!(
            "{}while ({}) {{\n",
            self.indent(),
            self.convert_expression(condition)?
        );

        self.add_indent();
        for stmt in body {
            output.push_str(&self.convert_statement(stmt)?);
        }
        self.rem_indent();

        output.push_str(&format!("{}}}\n", self.indent()));
        Ok(output)
    }

    fn convert_type(&mut self, ty: &DataType) -> Result<String, String> {
        match ty {
            DataType::Char => Ok("char".to_string()),
            DataType::UChar => Ok("u8".to_string()),
            DataType::Short => Ok("s16".to_string()),
            DataType::UShort => Ok("u16".to_string()),
            DataType::Int => Ok("s32".to_string()),
            DataType::UInt => Ok("u32".to_string()),
            DataType::Long => Ok("s32".to_string()),
            DataType::ULong => Ok("u32".to_string()),
            DataType::Quad => Ok("s64".to_string()),
            DataType::UQuad => Ok("u64".to_string()),
            DataType::Float => Ok("float".to_string()),
            DataType::Double => Ok("double".to_string()),
            DataType::Custom(name) => Ok(name.clone()),
            DataType::Struct(ident, body) => self.convert_struct(ident, body),
            DataType::Array(base_ty, _) => self.convert_type(base_ty),
        }
    }

    fn convert_expression(&mut self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Literal(lit) => Ok(lit.clone()),
            Expression::Identifier(var) => Ok(var.clone()),
            Expression::UnaryOp(op, right) => {
                Ok(format!("{}{}", op, self.convert_expression(right)?))
            }
            Expression::BinaryOp(left, op, right) => Ok(format!(
                "{} {} {}",
                self.convert_expression(left)?,
                op,
                self.convert_expression(right)?
            )),
            Expression::Call(name, args) => {
                let args_str = args
                    .iter()
                    .map(|a| self.convert_expression(a))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                Ok(format!("{}({})", name, args_str))
            }
            Expression::Cast(ty, expr) => Ok(format!(
                "({}) {}",
                self.convert_type(ty)?,
                self.convert_expression(expr)?
            )),
            Expression::FieldAccess(expr, field) => {
                Ok(format!("{}.{}", self.convert_expression(expr)?, field))
            }
            Expression::ArrayAccess(expr, index) => Ok(format!(
                "{}[{}]",
                self.convert_expression(expr)?,
                self.convert_expression(index)?
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
