use crate::ast::*;

pub struct HexPatConverter {
    indent: usize,
}

impl HexPatConverter {
    pub fn new() -> Self {
        HexPatConverter { indent: 0 }
    }

    pub fn convert(&self, template: &Template) -> Result<String, String> {
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

    fn convert_statement(&self, stmt: &Statement) -> Result<String, String> {
        match stmt {
            Statement::StructDef {
                ident: name,
                fields,
            } => {
                let mut output = format!("{}struct {} {{\n", self.indent_nested(), name);
                for content in fields {
                    output.push_str(&self.convert_struct_content(content)?);
                }
                output.push_str(&format!("{}}}\n", self.indent_nested()));
                Ok(output)
            }
            Statement::TypeDef { ident: name, ty } => self.convert_typedef(name, ty),
            Statement::EnumDef {
                ident: name,
                variants,
            } => self.convert_enum(name, variants),
            Statement::VarDecl {
                ident: name,
                ty,
                value,
                local: _,
            } => {
                let mut output = format!("{}", self.indent_str());

                output.push_str(&self.convert_type(ty)?);
                output.push_str(&format!(" {}", name));

                if let Some(expr) = value {
                    output.push_str(&format!(" = {}", self.convert_expression(expr)?));
                }

                output.push_str(";\n");
                Ok(output)
            }
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
            Statement::While { condition, body } => self.convert_while(condition, body),
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
            Statement::Return(_) => Ok(String::new()),
            Statement::Switch { expr, cases } => {
                let mut output = format!(
                    "{}match ({}) {{\n",
                    self.indent_str(),
                    self.convert_expression(expr)?
                );
                for (case_idx, case_body) in cases.iter().enumerate() {
                    if case_idx == cases.len() - 1 && !cases.is_empty() {
                        output.push_str(&format!("{}    (_):\n", self.indent_str()));
                    } else {
                        output.push_str(&format!("{}    ({}):\n", self.indent_str(), case_idx));
                    }
                    for stmt in case_body {
                        output.push_str(&self.convert_statement(stmt)?);
                    }
                    // output.push_str(&format!("{}        break;\n", self.indent_str()));
                }
                output.push_str(&format!("{}}}\n", self.indent_str()));
                Ok(output)
            }
        }
    }

    fn convert_struct_content(&self, content: &StructContent) -> Result<String, String> {
        match content {
            StructContent::Field(field) => {
                let type_str = self.convert_type(&field.ty)?;
                let mut output = format!(
                    "{}{} {}{}",
                    self.indent_str(),
                    type_str,
                    field.ident,
                    match &field.ty {
                        DataType::Array(_, size) => {
                            if let Some(size_expr) = size {
                                format!("[{}]", self.convert_expression(size_expr)?)
                            } else {
                                "[]".to_string()
                            }
                        }
                        _ => String::new(),
                    }
                );

                if let Some(cond) = &field.condition {
                    output.push_str(&format!(" if ({})", self.convert_expression(cond)?));
                }

                output.push_str(";\n");
                Ok(output)
            }
            StructContent::Statement(stmt) => self.convert_statement(stmt),
        }
    }

    fn convert_typedef(&self, name: &str, ty: &DataType) -> Result<String, String> {
        Ok(format!("using {} = {};\n\n", name, self.convert_type(ty)?))
    }

    fn convert_enum(
        &self,
        name: &str,
        variants: &[(String, Option<i64>)],
    ) -> Result<String, String> {
        let mut output = format!("enum {} : u32 {{\n", name);

        for (var_name, value) in variants {
            output.push_str(&format!("  {}", var_name));
            if let Some(v) = value {
                output.push_str(&format!(" = {}", v));
            }
            output.push_str(",\n");
        }

        output.push_str("};\n\n");
        Ok(output)
    }

    fn convert_if(
        &self,
        condition: &Expression,
        then_block: &[Statement],
        else_block: &Option<Vec<Statement>>,
    ) -> Result<String, String> {
        let mut output = format!(
            "{}if ({}) {{\n",
            self.indent_str(),
            self.convert_expression(condition)?
        );

        for stmt in then_block {
            output.push_str(&self.convert_statement(stmt)?);
        }

        output.push_str(&format!("{}}}", self.indent_str()));

        if let Some(else_stmts) = else_block {
            let else_if = matches!(else_stmts.get(0), Some(Statement::If { .. }));

            // match else_stmts.get(0) {
            //     Some(Statement::If { .. }) => output.push_str(" else "),
            //     _ => output.push_str(" else {\n"),
            // }
            output.push_str(if !else_if { " else {\n" } else { " else " });
            for stmt in else_stmts {
                output.push_str(&self.convert_statement(stmt)?);
            }
            if !else_if {
                output.push_str(&format!("{}}}", self.indent_str()));
            }
        }

        output.push_str("\n");
        Ok(output)
    }

    fn convert_while(&self, condition: &Expression, body: &[Statement]) -> Result<String, String> {
        let mut output = format!(
            "{}while ({}) {{\n",
            self.indent_str(),
            self.convert_expression(condition)?
        );

        for stmt in body {
            output.push_str(&self.convert_statement(stmt)?);
        }

        output.push_str(&format!("{}}}\n", self.indent_str()));
        Ok(output)
    }

    fn convert_type(&self, ty: &DataType) -> Result<String, String> {
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
            DataType::Array(base_ty, size) => {
                let base = self.convert_type(base_ty)?;
                Ok(base)
                // if let Some(size_expr) = size {
                //     Ok(format!("{}[{}]", base, self.convert_expression(size_expr)?))
                // } else {
                //     Ok(format!("{}[]", base))
                // }
            }
        }
    }

    fn convert_expression(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Literal(lit) => Ok(lit.clone()),
            Expression::Identifier(var) => Ok(var.clone()),
            Expression::UnaryOp(op, right) => {
                Ok(format!("{} {}", op, self.convert_expression(right)?))
            }
            Expression::BinaryOp(left, op, right) => Ok(format!(
                "{} {} {}",
                self.convert_expression(left)?,
                op,
                self.convert_expression(right)?
            )),
            Expression::FunctionCall(name, args) => {
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

    fn indent_str(&self) -> &'static str {
        "  "
    }

    fn indent_nested(&self) -> String {
        "  ".repeat(self.indent)
    }
}
