use std::fmt::Write;

use crate::ast::{
    node::{
        expression::Expr,
        module::Module,
        statement::Stmt,
        structure::Struct,
        type_signature::{Mutability, TypeSignature},
    },
    AST,
};

pub fn format_ast(ast: &AST) -> String {
    let mut out = String::new();
    format_module(&mut out, ast.inner_module());
    return out;
}

fn format_module(out: &mut String, module: &Module) {
    for st in &module.structs {
        format_struct(out, st);
        *out += "\n";
    }

    if !module.structs.is_empty() {
        *out += "\n";
    }

    for stmt in &module.stmts {
        format_stmt(out, stmt);
        *out += "\n";
    }
}

fn format_struct(_out: &mut String, _st: &Struct) {
    todo!()
}

fn format_expr(out: &mut String, expr: &Expr) {
    match expr {
        Expr::StringLiteral(string) => *out += string,
        Expr::NumberLiteral(num) => write!(*out, "{}", num).unwrap(),
        Expr::BoolLiteral(bool) => *out += if *bool { "true" } else { "false" },
        Expr::Function(_) => todo!(),
    }
}

fn format_stmt(out: &mut String, stmt: &Stmt) {
    match stmt {
        Stmt::VariableDecl(var_decl) => {
            *out += "let ";

            if var_decl.mutability == Mutability::Mutable {
                *out += "mut ";
            }

            *out += var_decl.name.value;

            if let Some(ref type_sig) = var_decl.type_sig {
                *out += ": ";
                format_type_sig(out, type_sig);
            }

            *out += " = ";
            format_expr(out, &var_decl.value);
        }
        Stmt::FunctionDecl(_func) => {
            todo!()
        }
        Stmt::Compound(stmts) => {
            for (i, stmt) in stmts.iter().enumerate() {
                format_stmt(out, stmt);
                if i < stmts.len() - 1 {
                    *out += "\n";
                }
            }
        }
    }
}

fn format_type_sig(out: &mut String, type_sig: &TypeSignature) {
    match type_sig {
        TypeSignature::Base(base) => *out += base.value,
        TypeSignature::Function {
            args: _,
            return_type: _,
        } => todo!(),
        TypeSignature::Reference(_) => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_ast;

    use super::*;

    #[test]
    fn test_formatting() {
        let ast = parse_ast("let  mut value: Number    =212.2; let x = 2").unwrap();
        assert_eq!(
            format_ast(&ast),
            "let mut value: Number = 212.2\n\
            let x = 2\n"
        );
    }
}
