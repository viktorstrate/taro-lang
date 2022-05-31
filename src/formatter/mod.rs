use crate::ast::{Expr, Mutability, Stmt, TypeSignature};

pub struct Formatter {}

impl Formatter {
    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLiteral(string) => string.to_string(),
            Expr::NumberLiteral(num) => format!("{num}"),
            Expr::BoolLiteral(bool) => if *bool { "true" } else { "false" }.to_string(),
        }
    }

    fn format_stmt(&self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDecl(var_decl) => {
                let mut result = "let".to_string();

                if var_decl.mutability == Mutability::Mutable {
                    result += " mut";
                }

                result += " ";
                result += var_decl.name.as_str();

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

    fn format_type_sig(&self, type_sig: &TypeSignature) -> String {
        match type_sig {
            TypeSignature::Base(base) => base.as_str().to_string(),
            TypeSignature::Function(_, _, _) => todo!(),
            TypeSignature::Reference(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::*;

    #[test]
    fn test_formatting() {
        let (_, ast) =
            parser::statements::statement("let  mut value: Number    =212; let x = 2").unwrap();
        assert_eq!(
            Formatter {}.format_stmt(&ast),
            "let mut value: Number = 212\n\
            let x = 2"
        );
    }
}
