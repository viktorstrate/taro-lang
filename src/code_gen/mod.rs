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
    symbols::{symbol_table::symbol_table_zipper::SymbolTableZipper, symbol_table::SymbolValue},
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

fn format_struct<'a, W: Write>(ctx: &mut CodeGenCtx<'a, W>, st: &Struct<'a>) -> CodeGenResult {
    ctx.write("function ")?;
    ctx.write_ident(&st.name)?;

    ctx.symbols.enter_scope(st.name.clone()).unwrap();

    ctx.write(" (")?;

    format_with_separator(ctx, ", ", st.attrs.iter(), |ctx, attr| {
        ctx.write_ident(&attr.name)
    })?;

    ctx.write(") {\n")?;

    format_with_separator(ctx, ";\n", st.attrs.iter(), |ctx, attr| {
        ctx.write("this.")?;
        ctx.write_ident(&attr.name)?;
        ctx.write(" = ")?;
        ctx.write_ident(&attr.name)?;

        if let Some(default) = &attr.default_value {
            ctx.write(" ?? ")?;
            format_expr(ctx, default)?;
        }

        Ok(())
    })?;

    ctx.write("}")?;

    ctx.symbols.exit_scope().unwrap();

    Ok(())
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
    ctx.symbols.visit_next_symbol();

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

    ctx.symbols
        .enter_scope(func.name.clone())
        .expect("function scope should exist");

    format_func_args(ctx, &func.args)?;

    ctx.write(" {")?;

    format_stmt(ctx, &func.body)?;

    ctx.symbols.exit_scope().unwrap();

    ctx.write("}")?;

    Ok(())
}

fn format_expr<'a, W: Write>(ctx: &mut CodeGenCtx<'a, W>, expr: &Expr<'a>) -> CodeGenResult {
    match expr {
        Expr::StringLiteral(str) => ctx.write(str),
        Expr::NumberLiteral(num) => ctx.write_fmt(format_args!("{}", num)),
        Expr::BoolLiteral(val) => ctx.write(if *val == true { "true" } else { "false" }),
        Expr::Function(func) => {
            ctx.symbols
                .enter_scope(func.name.clone())
                .expect("function scope should exist");

            format_func_args(ctx, &func.args)?;
            ctx.write(" => {")?;

            format_stmt(ctx, &func.body)?;

            ctx.symbols.exit_scope().unwrap();

            ctx.write("}")
        }
        Expr::FunctionCall(call) => {
            format_expr(ctx, &call.func)?;
            ctx.write("(")?;
            format_with_separator(ctx, ", ", call.params.iter(), format_expr)?;
            ctx.write(")")
        }
        Expr::Identifier(ident) => ctx.write_ident(ident),
        Expr::StructInit(st_init) => {
            ctx.write("new ")?;
            ctx.write_ident(&st_init.name)?;
            ctx.write("(")?;

            let st = ctx.symbols.lookup(&st_init.name).unwrap();
            let st = match st {
                SymbolValue::StructDecl(st) => st,
                _ => unreachable!(),
            };

            let attr_names = st
                .attrs
                .iter()
                .map(|attr| attr.name.clone())
                .collect::<Vec<_>>();

            format_with_separator(ctx, ", ", attr_names.iter(), |ctx, attr_name| {
                let attr_val = st_init.values.iter().find(|val| val.name == *attr_name);
                if let Some(val) = attr_val {
                    format_expr(ctx, &val.value)
                } else {
                    ctx.write("null")
                }
            })?;

            ctx.write(")")
        }
        Expr::EscapeBlock(block) => ctx.write(block.content),
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

    #[test]
    fn test_struct() {
        let ast = final_codegen("struct Test { let defaultVal = 123; let mut noDefault: Boolean }");
        assert_eq!(ast.unwrap(), "function struct_Test (defaultVal, noDefault) {\nthis.defaultVal = defaultVal ?? 123;\nthis.noDefault = noDefault}\n");
    }
}
