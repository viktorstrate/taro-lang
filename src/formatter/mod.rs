use crate::ast::{Expr, Mutability, Stmt, TypeSignature, AST};

#[derive(Default)]
pub struct Formatter {}

impl Formatter {
    pub fn format_ast(&self, ast: &AST) -> String {
        self.format_stmt(ast.inner_stmt())
    }

    pub fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLiteral(string) => string.to_string(),
            Expr::NumberLiteral(num) => format!("{num}"),
            Expr::BoolLiteral(bool) => if *bool { "true" } else { "false" }.to_string(),
        }
    }

    pub fn format_stmt(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDecl(var_decl) => {
                let mut result = "let".to_string();

                if var_decl.mutability == Mutability::Mutable {
                    result += " mut";
                }

                result += " ";
                result += var_decl.name.value;

                if let Some(ref type_sig) = var_decl.type_sig {
                    result += ": ";
                    result += self.format_type_sig(type_sig).as_str();
                }

                result += " = ";
                result += self.format_expr(&var_decl.value).as_str();

                result
            }
            Stmt::Compound(stmts) => stmts
                .iter()
                .map(|stmt| self.format_stmt(stmt))
                .intersperse("\n".to_string())
                .collect::<String>(),
        }
    }

    pub fn format_type_sig(&self, type_sig: &TypeSignature) -> String {
        match type_sig {
            TypeSignature::Base(base) => base.value.to_string(),
            TypeSignature::Function(_, _, _) => todo!(),
            TypeSignature::Reference(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_ast;

    use super::*;

    #[test]
    fn test_formatting() {
        let ast = parse_ast("let  mut value: Number    =212; let x = 2").unwrap();
        assert_eq!(
            Formatter::default().format_ast(&ast),
            "let mut value: Number = 212\n\
            let x = 2"
        );
    }
}
