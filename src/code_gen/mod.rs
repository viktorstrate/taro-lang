use std::io::{BufWriter, Write};

use crate::{
    ast::{
        node::{
            expression::Expr,
            function::{Function, FunctionArg},
            identifier::Ident,
            module::Module,
            statement::{Stmt, VarDecl},
            structure::Struct,
            type_signature::Mutability,
        },
        AST,
    },
    symbols::symbol_table_zipper::SymbolTableZipper,
};

pub fn format_ast<'a, W: Write>(
    writer: W,
    ast: &AST<'a>,
    symbols: SymbolTableZipper<'a>,
) -> Result<SymbolTableZipper<'a>, std::io::Error> {
    let mut ctx = CodeGenCtx {
        writer: BufWriter::new(writer),
        symbols,
    };
    format_module(&mut ctx, ast.inner_module())?;
    Ok(ctx.symbols)
}

struct CodeGenCtx<'a, W: Write> {
    writer: BufWriter<W>,
    symbols: SymbolTableZipper<'a>,
}

type CodeGenResult = std::io::Result<()>;

impl<'a, W: Write> CodeGenCtx<'a, W> {
    fn write(&mut self, s: &str) -> CodeGenResult {
        self.writer.write(s.as_bytes())?;
        Ok(())
    }

    fn write_ident(&mut self, ident: &Ident<'a>) -> CodeGenResult {
        ident.write(&mut self.writer, &self.symbols)
    }
}

impl<'a, W: Write> Write for CodeGenCtx<'a, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

fn format_module<'a, W: Write>(ctx: &mut CodeGenCtx<'a, W>, module: &Module<'a>) -> CodeGenResult {
    format_with_separator(ctx, "\n", module.stmts.iter(), format_stmt)?;
    ctx.write("\n")
}

fn format_struct<W: Write>(ctx: &mut CodeGenCtx<W>, st: &Struct) -> CodeGenResult {
    ctx.write("INSERT STRUCT HERE")
}

fn format_stmt<'a, W: Write>(ctx: &mut CodeGenCtx<'a, W>, stmt: &Stmt<'a>) -> CodeGenResult {
    match stmt {
        Stmt::VariableDecl(var_decl) => format_var_decl(ctx, var_decl),
        Stmt::FunctionDecl(func_decl) => format_func_decl(ctx, func_decl),
        Stmt::Compound(stmts) => format_with_separator(ctx, "\n", stmts.iter(), format_stmt),
        Stmt::Expression(expr) => {
            format_expr(ctx, expr)?;
            ctx.write(";")
        }
        Stmt::StructDecl(st) => format_struct(ctx, st),
        Stmt::Return(expr) => {
            ctx.write("return ")?;
            format_expr(ctx, expr)?;
            ctx.write(";")
        }
    }
}

fn format_var_decl<'a, W: Write>(
    ctx: &mut CodeGenCtx<'a, W>,
    var_decl: &VarDecl<'a>,
) -> CodeGenResult {
    if var_decl.mutability == Mutability::Mutable {
        ctx.write("let ")?;
    } else {
        ctx.write("const ")?;
    }

    ctx.write_ident(&var_decl.name)?;
    ctx.write(" = ")?;
    format_expr(ctx, &var_decl.value)?;
    ctx.write(";")
}

fn format_func_decl<'a, W: Write>(
    ctx: &mut CodeGenCtx<'a, W>,
    func: &Function<'a>,
) -> CodeGenResult {
    ctx.write("function ")?;
    ctx.write_ident(&func.name)?;

    format_func_args(ctx, &func.args)?;

    ctx.write(" {")?;
    format_stmt(ctx, &func.body)?;
    ctx.write("}")?;

    Ok(())
}

fn format_expr<'a, W: Write>(ctx: &mut CodeGenCtx<'a, W>, expr: &Expr<'a>) -> CodeGenResult {
    match expr {
        Expr::StringLiteral(str) => ctx.write(str),
        Expr::NumberLiteral(num) => ctx.write_fmt(format_args!("{}", num)),
        Expr::BoolLiteral(val) => ctx.write(if *val == true { "true" } else { "false" }),
        Expr::Function(func) => {
            format_func_args(ctx, &func.args)?;
            ctx.write(" => {")?;
            format_stmt(ctx, &func.body)?;
            ctx.write("}")
        }
        Expr::FunctionCall(call) => {
            format_expr(ctx, &call.func)?;
            ctx.write("(")?;
            format_with_separator(ctx, ", ", call.params.iter(), format_expr)?;
            ctx.write(")")
        }
        Expr::Identifier(ident) => ctx.write_ident(ident),
    }
}

fn format_func_args<'a, W: Write>(
    ctx: &mut CodeGenCtx<'a, W>,
    args: &Vec<FunctionArg<'a>>,
) -> CodeGenResult {
    ctx.write("(")?;
    format_with_separator(ctx, ", ", args.iter(), |ctx, arg| {
        ctx.write_ident(&arg.name)
    })?;
    ctx.write(")")
}

fn format_with_separator<W, I, T, F>(
    writer: &mut W,
    sep: &str,
    items: I,
    format: F,
) -> std::io::Result<()>
where
    W: Write,
    F: Fn(&mut W, T) -> std::io::Result<()>,
    I: ExactSizeIterator<Item = T>,
{
    let len = items.len() as isize;
    for (i, elem) in items.enumerate() {
        format(writer, elem)?;
        if (i as isize) < len - 1 {
            writer.write_all(sep.as_bytes())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ast::test_utils::utils::{final_codegen, FinalAstError};

    #[test]
    fn test_let_assign_simple() {
        assert_eq!(
            final_codegen("let val: Number = 23.4").unwrap(),
            "const val = 23.4;\n"
        )
    }

    #[test]
    fn test_func_call() {
        assert_eq!(
            final_codegen("func f() {}; f()").unwrap(),
            "function f() {}\nf();\n"
        );
    }

    #[test]
    fn test_assign_func_call() {
        let ast1 = final_codegen("func f() -> Boolean { return true }; let x: Boolean = f()");
        let ast2 = final_codegen("let f = () { return true }; let x: Boolean = f()");
        assert_matches!(ast1, Ok(_));
        assert_matches!(ast2, Ok(_));
        assert_eq!(
            ast1.unwrap(),
            "function f() {return true;}\nconst x = f();\n"
        );
        assert_eq!(
            ast2.unwrap(),
            "const f = () => {return true;};\nconst x = f();\n"
        );
    }

    #[test]
    fn test_assign_func_call_mismatched_types() {
        let ast = final_codegen("func f() { return 123 }; let x: Boolean = f()");
        assert_matches!(ast, Err(FinalAstError::TypeCheck(_)));
    }
}
