use std::io::{BufWriter, Write};

use crate::{
    ir::{
        context::IrCtx,
        node::{
            expression::Expr,
            function::{Function, FunctionArg},
            identifier::{Ident, IdentValue, ResolvedIdentValue},
            module::Module,
            statement::{Stmt, StmtBlock, VarDecl},
            structure::Struct,
            type_signature::Mutability,
            NodeRef,
        },
        IR,
    },
    symbols::symbol_table::symbol_table_zipper::SymbolTableZipper,
};

pub fn format_ir<'a, 'ctx, W: Write>(
    writer: &mut W,
    ctx: &mut IrCtx<'a>,
    mut symbols: SymbolTableZipper<'a>,
    ir: &mut IR<'a>,
) -> Result<SymbolTableZipper<'a>, std::io::Error> {
    symbols.reset(&ctx);
    let mut ctx = CodeGenCtx {
        writer: BufWriter::new(writer),
        symbols,
        ctx,
    };
    format_module(&mut ctx, &ir.0)?;
    Ok(ctx.symbols)
}

pub struct CodeGenCtx<'a, 'ctx, W: Write> {
    pub writer: BufWriter<W>,
    pub symbols: SymbolTableZipper<'a>,
    pub ctx: &'ctx mut IrCtx<'a>,
}

type CodeGenResult = std::io::Result<()>;

impl<'a, 'ctx, W: Write> CodeGenCtx<'a, 'ctx, W> {
    fn write(&mut self, s: &str) -> CodeGenResult {
        self.writer.write(s.as_bytes())?;
        Ok(())
    }

    fn write_ident(&mut self, ident: Ident<'a>) -> CodeGenResult {
        match &self.ctx[ident] {
            IdentValue::Resolved(resolved_ident) => match resolved_ident {
                ResolvedIdentValue::Named { def_span: _, name } => self.write(name),
                ResolvedIdentValue::BuiltinType(builtin) => self.write(builtin.name()),
                ResolvedIdentValue::Anonymous => {
                    panic!("Anonymous identifiers should not be written")
                }
            },
            IdentValue::Unresolved(id) => {
                unreachable!(
                    "all identifiers should be resolved by now: (PARENT: {:?}, IDENT: {id:?})",
                    ident.parent
                )
            }
        }
    }
}

impl<'a, 'ctx, W: Write> Write for CodeGenCtx<'a, 'ctx, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

fn format_module<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    module: &Module<'a>,
) -> CodeGenResult {
    format_stmt_block(gen, module.stmt_block)?;
    gen.write("\n")
}

fn format_struct<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    st: NodeRef<'a, Struct<'a>>,
) -> CodeGenResult {
    gen.write("function ")?;
    gen.write_ident(*gen.ctx[st].name)?;

    gen.symbols
        .enter_scope(&gen.ctx, *gen.ctx[st].name)
        .unwrap();

    gen.write(" (")?;

    format_with_separator(
        gen,
        ", ",
        gen.ctx[st].attrs.clone().into_iter(),
        |gen, attr| gen.write_ident(*gen.ctx[attr].name),
    )?;

    gen.write(") {\n")?;

    format_with_separator(
        gen,
        ";\n",
        gen.ctx[st].attrs.clone().into_iter(),
        |gen, attr| {
            gen.write("this.")?;
            gen.write_ident(*gen.ctx[attr].name)?;
            gen.write(" = ")?;
            gen.write_ident(*gen.ctx[attr].name)?;

            if let Some(default) = gen.ctx[attr].default_value {
                gen.write(" ?? ")?;
                format_expr(gen, default)?;
            }

            Ok(())
        },
    )?;

    gen.write("}")?;

    gen.symbols.exit_scope(&gen.ctx).unwrap();

    Ok(())
}

fn format_stmt_block<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    stmt_block: NodeRef<'a, StmtBlock<'a>>,
) -> CodeGenResult {
    format_with_separator(
        gen,
        "\n",
        gen.ctx[stmt_block].0.clone().into_iter(),
        format_stmt,
    )
}

fn format_stmt<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    stmt: NodeRef<'a, Stmt<'a>>,
) -> CodeGenResult {
    match gen.ctx[stmt].clone() {
        Stmt::VariableDecl(var_decl) => format_var_decl(gen, var_decl),
        Stmt::FunctionDecl(func_decl) => format_func_decl(gen, func_decl),
        Stmt::Expression(expr) => {
            format_expr(gen, expr)?;
            gen.write(";")
        }
        Stmt::StructDecl(st) => format_struct(gen, st),
        Stmt::EnumDecl(_) => Ok(()),
        Stmt::Return(expr) => {
            gen.write("return ")?;
            format_expr(gen, expr)?;
            gen.write(";")
        }
    }
}

fn format_var_decl<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    var_decl: NodeRef<'a, VarDecl<'a>>,
) -> CodeGenResult {
    gen.symbols.visit_next_symbol(&gen.ctx);

    if gen.ctx[var_decl].mutability == Mutability::Mutable {
        gen.write("let ")?;
    } else {
        gen.write("const ")?;
    }

    gen.write_ident(*gen.ctx[var_decl].name)?;
    gen.write(" = ")?;
    format_expr(gen, gen.ctx[var_decl].value)?;
    gen.write(";")
}

fn format_func_decl<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    func: NodeRef<'a, Function<'a>>,
) -> CodeGenResult {
    let func_name = *gen.ctx[func].name;

    gen.write("function ")?;
    gen.write_ident(func_name)?;

    gen.symbols
        .enter_scope(&gen.ctx, func_name)
        .expect("function scope should exist");

    format_func_args(gen, gen.ctx[func].args.clone())?;

    gen.write(" {")?;

    format_stmt_block(gen, gen.ctx[func].body)?;

    gen.symbols.exit_scope(&gen.ctx).unwrap();

    gen.write("}")?;

    Ok(())
}

fn format_expr<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    expr: NodeRef<'a, Expr<'a>>,
) -> CodeGenResult {
    match gen.ctx[expr].clone() {
        Expr::StringLiteral(str, _) => {
            gen.write("\"")?;
            gen.write(str)?;
            gen.write("\"")
        }
        Expr::NumberLiteral(num, _) => gen.write_fmt(format_args!("{}", num)),
        Expr::BoolLiteral(val, _) => gen.write(if val == true { "true" } else { "false" }),
        Expr::Function(func) => {
            gen.symbols
                .enter_scope(&gen.ctx, *gen.ctx[func].name)
                .expect("function scope should exist");

            format_func_args(gen, gen.ctx[func].args.clone())?;
            gen.write(" => {")?;

            format_stmt_block(gen, gen.ctx[func].body)?;

            gen.symbols.exit_scope(&gen.ctx).unwrap();

            gen.write("}")
        }
        Expr::FunctionCall(call) => {
            println!("GEN FUNC CALL");
            format_expr(gen, gen.ctx[call].func)?;
            gen.write("(")?;
            format_with_separator(
                gen,
                ", ",
                gen.ctx[call].params.clone().into_iter(),
                format_expr,
            )?;
            gen.write(")")
        }
        Expr::Identifier(ident) => gen.write_ident(*ident),
        Expr::StructInit(st_init) => {
            gen.symbols
                .enter_scope(&gen.ctx, *gen.ctx[st_init].scope_name)
                .expect("struct init scope should exist");
            gen.write("new ")?;
            gen.write_ident(*gen.ctx[st_init].struct_name)?;
            gen.write("(")?;

            let st = gen
                .symbols
                .lookup(&gen.ctx, *gen.ctx[st_init].struct_name)
                .unwrap()
                .unwrap_struct(&gen.ctx);

            let attr_names = gen.ctx[st]
                .attrs
                .iter()
                .map(|attr| *gen.ctx[*attr].name)
                .collect::<Vec<_>>();

            format_with_separator(gen, ", ", attr_names.iter(), |gen, attr_name| {
                let attr_val = gen.ctx[st_init]
                    .values
                    .iter()
                    .find(|val| *gen.ctx[**val].name == *attr_name);
                if let Some(val) = attr_val {
                    format_expr(gen, gen.ctx[*val].value)
                } else {
                    gen.write("null")
                }
            })?;

            gen.write(")")?;
            gen.symbols.exit_scope(&gen.ctx).unwrap();
            Ok(())
        }
        Expr::StructAccess(st_access) => {
            format_expr(gen, gen.ctx[st_access].struct_expr)?;
            gen.write(".")?;

            gen.write_ident(gen.ctx[st_access].attr_name)
        }
        Expr::EscapeBlock(block) => gen.write(gen.ctx[block].content),
        Expr::Assignment(asg) => {
            format_expr(gen, gen.ctx[asg].lhs)?;
            gen.write(" = ")?;
            format_expr(gen, gen.ctx[asg].rhs)
        }
        Expr::Tuple(tup) => {
            gen.write("[")?;
            for (i, val) in gen.ctx[tup].values.clone().into_iter().enumerate() {
                format_expr(gen, val)?;

                if i < gen.ctx[tup].values.len() - 1 {
                    gen.write(", ")?;
                }
            }
            gen.write("]")
        }
        Expr::TupleAccess(tup_acc) => {
            format_expr(gen, gen.ctx[tup_acc].tuple_expr)?;
            gen.write("[")?;
            gen.write(gen.ctx[tup_acc].attr.to_string().as_str())?;
            gen.write("]")
        }
        Expr::EnumInit(enm_init) => {
            gen.write("[")?;
            let enm_name = gen.ctx[enm_init].enum_name;

            let enm = gen
                .symbols
                .lookup(gen.ctx, enm_name)
                .expect("Symbol should exist")
                .unwrap_enum(&gen.ctx);

            let (idx, _enm_val) = enm
                .lookup_value(&gen.ctx, gen.ctx[enm_init].enum_value)
                .expect("Expected to find enum value");

            gen.write(format!("{idx}, ").as_str())?;

            gen.write("[")?;

            format_with_separator(
                gen,
                ", ",
                gen.ctx[enm_init].items.clone().into_iter(),
                |gen, item| format_expr(gen, item),
            )?;

            gen.write("]")?;

            gen.write("]")
        }
        Expr::UnresolvedMemberAccess(_) => {
            unreachable!("Unresolved member access should have been handled by now")
        }
    }
}

fn format_func_args<'a, 'ctx, W: Write>(
    gen: &mut CodeGenCtx<'a, 'ctx, W>,
    args: Vec<NodeRef<'a, FunctionArg<'a>>>,
) -> CodeGenResult {
    gen.write("(")?;
    format_with_separator(gen, ", ", args.iter(), |gen, arg| {
        gen.write_ident(*gen.ctx[*arg].name)
    })?;
    gen.write(")")
}

fn format_with_separator<W, I, T>(
    writer: &mut W,
    sep: &str,
    items: I,
    format: impl Fn(&mut W, T) -> std::io::Result<()>,
) -> std::io::Result<()>
where
    W: Write,
    I: ExactSizeIterator<Item = T>,
    T: Copy,
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

    use crate::ir::test_utils::utils::final_codegen;

    #[test]
    fn test_let_declare_simple() {
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
        let output1 = final_codegen("func f() -> Boolean { return true }; let x: Boolean = f()");
        let output2 = final_codegen("let f = () { return true }; let x: Boolean = f()");
        assert_matches!(output1, Ok(_));
        assert_matches!(output2, Ok(_));
        assert_eq!(
            output1.unwrap(),
            "function f() {return true;}\nconst x = f();\n"
        );
        assert_eq!(
            output2.unwrap(),
            "const f = () => {return true;};\nconst x = f();\n"
        );
    }

    #[test]
    fn test_func_call_inside_struct() {
        let output = final_codegen(
            "struct Foo {\n\
                let bar: () -> Void\n\
              }

              let x = Foo { bar: () {} }\n\
              x.bar()\n\
              ",
        );
        assert_eq!(
            output.unwrap(),
            "function Foo (bar) {\n\
            this.bar = bar}\n\
            const x = new Foo(() => {});\n\
            x.bar();\n"
        )
    }

    #[test]
    fn test_struct() {
        let output = final_codegen(
            "struct Test { let defaultVal = 123; var noDefault: Boolean }\n\
            let testVar = Test { noDefault: false }
            let val: Number = testVar.defaultVal
        ",
        );
        assert_eq!(
            output.unwrap(),
            "function Test (defaultVal, noDefault) {\n\
            this.defaultVal = defaultVal ?? 123;\n\
            this.noDefault = noDefault}\n\
            const testVar = new Test(null, false);\n\
            const val = testVar.defaultVal;\n"
        );
    }

    #[test]
    fn test_tuple() {
        let output = final_codegen(
            "let val: (Boolean, Number) = (true, 42)\n\
            let val2: Number = val.1",
        );
        assert_eq!(
            output.unwrap(),
            "const val = [true, 42];\n\
            const val2 = val[1];\n"
        );
    }

    #[test]
    fn test_enum() {
        let output = final_codegen(
            "enum IPAddress {\n\
                v4(Number, Number, Number, Number)\n\
                v6(String)\n\
              }\n
              let ipValue = IPAddress.v4(192, 168, 0, 1)",
        );
        assert_eq!(
            output.unwrap(),
            "\nconst ipValue = [0, [192, 168, 0, 1]];\n"
        );
    }

    #[test]
    fn test_implicit_enum() {
        let output = final_codegen(
            "enum IPAddress {\n\
                v4(Number, Number, Number, Number)\n\
                v6(String)\n\
              }\n
              let ipValue: IPAddress = .v4(192, 168, 0, 1)",
        );
        assert_eq!(
            output.unwrap(),
            "\nconst ipValue = [0, [192, 168, 0, 1]];\n"
        );
    }
}
