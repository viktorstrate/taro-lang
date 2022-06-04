use std::fmt::Write;

use crate::ast::{
    node::{
        expression::Expr,
        function::{FunctionArg, FunctionDecl},
        module::Module,
        statement::{Stmt, VarDecl},
        structure::Struct,
        type_signature::Mutability,
    },
    AST,
};

pub fn ast_to_js(ast: &AST) -> String {
    let mut result = String::new();
    format_module(&mut result, ast.inner_module());
    result
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
        *out += ";\n";
    }
}

fn format_struct(out: &mut String, st: &Struct) {
    writeln!(*out, "INSERT STRUCT {} HERE", st.name.value).unwrap();
}

fn format_stmt(out: &mut String, stmt: &Stmt) {
    match stmt {
        Stmt::VariableDecl(var_decl) => format_var_decl(out, var_decl),
        Stmt::FunctionDecl(func_decl) => format_func_decl(out, func_decl),
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

fn format_func_decl(out: &mut String, func: &FunctionDecl) {
    *out += "function ";
    *out += func.name.value;

    format_func_args(out, &func.args);

    *out += " {";
    format_stmt(out, &func.body);
    *out += "}";
}

fn format_expr(out: &mut String, expr: &Expr) {
    match expr {
        Expr::StringLiteral(str) => write!(*out, "\"{str}\"").unwrap(),
        Expr::NumberLiteral(num) => write!(*out, "{num}").unwrap(),
        Expr::BoolLiteral(val) => *out += if *val == true { "true" } else { "false" },
        Expr::Function(func) => {
            format_func_args(out, &func.args);
            *out += " => {";
            format_stmt(out, &func.body);
            *out += "}";
        }
    };
}

fn format_func_args(out: &mut String, args: &Vec<FunctionArg>) {
    *out += "(";

    for (i, arg) in args.iter().enumerate() {
        *out += arg.name.value;
        if args.len() < i - 1 {
            *out += ", ";
        }
    }

    *out += ")";
}

#[cfg(test)]
mod tests {
    use super::ast_to_js;
    use crate::parser::parse_ast;

    #[test]
    fn test_code_gen() {
        let ast = parse_ast("let val = 23.4").unwrap();
        assert_eq!(ast_to_js(&ast), "const val = 23.4;\n")
    }
}
