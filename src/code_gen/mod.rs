use crate::ast::{Expr, Mutability, Stmt, VarDecl, AST};
use std::fmt::Write;

pub fn ast_to_js(ast: &AST) -> String {
    let mut result = String::new();
    format_stmt(&mut result, ast.inner_stmt());
    result
}

fn format_stmt(out: &mut String, stmt: &Stmt) {
    match stmt {
        Stmt::VarDecl(var_decl) => format_var_decl(out, var_decl),
        Stmt::Compound(stmts) => {
            for stmt in stmts {
                format_stmt(out, stmt);
                *out += ";\n";
            }
        }
    }
}

fn format_var_decl(out: &mut String, var_decl: &VarDecl) {
    if var_decl.mutability == Mutability::Mutable {
        *out += "let ";
    } else {
        *out += "const ";
    }

    *out += var_decl.name.value;
    *out += " = ";
    format_expr(out, &var_decl.value);
}

fn format_expr(out: &mut String, expr: &Expr) {
    match expr {
        Expr::StringLiteral(str) => write!(*out, "\"{str}\"").unwrap(),
        Expr::NumberLiteral(num) => write!(*out, "{num}").unwrap(),
        Expr::BoolLiteral(val) => *out += if *val == true { "true" } else { "false" },
    };
}

#[cfg(test)]
mod tests {
    use super::ast_to_js;
    use crate::parser::parse_ast;

    #[test]
    fn test_code_gen() {
        let ast = parse_ast("let val = 23.4").unwrap();
        assert_eq!(ast_to_js(&ast), "const val = 23.4")
    }
}
