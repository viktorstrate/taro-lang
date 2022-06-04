use std::fmt::Write;

use crate::ast::{
    node::{
        expression::Expr,
        function::{FuncDecl, FunctionArg},
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
    format_with_separator(out, "\n", module.stmts.iter(), format_stmt);
    *out += "\n";
}

fn format_struct(out: &mut String, st: &Struct) {
    writeln!(*out, "INSERT STRUCT {} HERE", st.name.value).unwrap();
}

fn format_stmt(out: &mut String, stmt: &Stmt) {
    match stmt {
        Stmt::VariableDecl(var_decl) => format_var_decl(out, var_decl),
        Stmt::FunctionDecl(func_decl) => format_func_decl(out, func_decl),
        Stmt::Compound(stmts) => {
            format_with_separator(out, "\n", stmts.iter(), format_stmt);
        }
        Stmt::Expression(expr) => format_expr(out, expr),
        Stmt::StructDecl(st) => format_struct(out, st),
        Stmt::Return(expr) => {
            *out += "return ";
            format_expr(out, expr);
            *out += ";";
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
    *out += ";";
}

fn format_func_decl(out: &mut String, func: &FuncDecl) {
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
        Expr::FunctionCall(call) => {
            format_expr(out, &call.func);
            *out += "(";
            format_with_separator(out, ", ", call.params.iter(), |out, param| {
                format_expr(out, param);
            });
            *out += ");";
        }
        Expr::Identifier(ident) => *out += ident.value,
    };
}

fn format_func_args(out: &mut String, args: &Vec<FunctionArg>) {
    *out += "(";
    format_with_separator(out, ", ", args.iter(), |out, arg| {
        *out += arg.name.value;
    });
    *out += ")";
}

fn format_with_separator<I, T, F>(out: &mut String, sep: &str, items: I, format: F)
where
    F: Fn(&mut String, T),
    I: ExactSizeIterator<Item = T>,
{
    let len = items.len() as isize;
    for (i, elem) in items.enumerate() {
        format(out, elem);
        if (i as isize) < len - 1 {
            *out += sep;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::ast_to_js;
    use crate::ast::test_utils::utils::final_ast;

    #[test]
    fn test_let_assign_simple() {
        let ast = final_ast("let val: Number = 23.4").unwrap();
        assert_eq!(ast_to_js(&ast), "const val = 23.4;\n")
    }

    #[test]
    fn test_func_call() {
        let ast = final_ast("func f() {}; f()");
        assert_matches!(ast, Ok(_));
        assert_eq!(ast_to_js(&ast.unwrap()), "function f() {}\nf();\n")
    }
}
