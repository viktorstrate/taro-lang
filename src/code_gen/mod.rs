use std::io::{BufWriter, Write};

use crate::{
    ast::{
        node::{
            expression::Expr,
            function::{Function, FunctionArg},
            identifier::{Ident, IdentValue},
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
        Stmt::EnumDecl(_) => Ok(()),
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
        Expr::StringLiteral(str) => {
            ctx.write("\"")?;
            ctx.write(str)?;
            ctx.write("\"")
        }
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
            ctx.symbols
                .enter_scope(st_init.scope_name.clone())
                .expect("struct init scope should exist");
            ctx.write("new ")?;
            ctx.write_ident(&st_init.struct_name)?;
            ctx.write("(")?;

            let st = ctx.symbols.lookup(&st_init.struct_name).unwrap();
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

            ctx.write(")")?;
            ctx.symbols.exit_scope().unwrap();
            Ok(())
        }
        Expr::StructAccess(st_access) => {
            format_expr(ctx, &*st_access.struct_expr)?;
            ctx.write(".")?;

            // attribute identifier cannot be looked up since it is under the scope of the struct
            // ctx.write_ident(&st_access.attr_name)

            match &st_access.attr_name.value {
                IdentValue::Named(attr_str) => ctx.write(attr_str),
                _ => unreachable!(),
            }
        }
        Expr::EscapeBlock(block) => ctx.write(block.content),
        Expr::Assignment(asg) => {
            format_expr(ctx, &asg.lhs)?;
            ctx.write(" = ")?;
            format_expr(ctx, &asg.rhs)
        }
        Expr::Tuple(tup) => {
            ctx.write("[")?;
            for (i, val) in tup.values.iter().enumerate() {
                format_expr(ctx, val)?;

                if i < tup.values.len() - 1 {
                    ctx.write(", ")?;
                }
            }
            ctx.write("]")
        }
        Expr::TupleAccess(tup_acc) => {
            format_expr(ctx, tup_acc.tuple_expr.as_ref())?;
            ctx.write("[")?;
            ctx.write(tup_acc.attr.to_string().as_str())?;
            ctx.write("]")
        }
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
        let ast = final_codegen(
            "struct Test { let defaultVal = 123; let mut noDefault: Boolean }\
            let testVar = Test { noDefault: false }
            let val: Number = testVar.defaultVal
        ",
        );
        assert_eq!(
            ast.unwrap(),
            "function Test (defaultVal, noDefault) {\n\
            this.defaultVal = defaultVal ?? 123;\n\
            this.noDefault = noDefault}\n\
            const testVar = new Test(null, false);\n\
            const val = testVar.defaultVal;\n"
        );
    }

    #[test]
    fn test_tuple() {
        let ast = final_codegen(
            "let val: (Boolean, Number) = (true, 42)\
            let val2: Number = val.1",
        );
        assert_eq!(
            ast.unwrap(),
            "const val = [true, 42];\n\
            const val2 = val[1];\n"
        );
    }

    // #[test]
    // fn test_enum() {
    //     let ast = final_codegen(
    //         "enum IPAddress {\n\
    //             v4(Number, Number, Number, Number)\n\
    //             v6(String)\n\
    //           }\n
    //           let ipValue: IPAddress = .v4(192, 168, 0, 1)",
    //     );
    //     assert_eq!(ast.unwrap(), "\n");
    // }
}
