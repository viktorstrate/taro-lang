use crate::{
    ast::AST,
    ir::{
        late_init::LateInit,
        node::{
            identifier::IdentParent,
            statement::Stmt,
            type_signature::{TypeSignature, TypeSignatureContext, TypeSignatureParent},
        },
    },
};

use super::{
    context::IrCtx,
    node::{
        assignment::Assignment,
        enumeration::{Enum, EnumValue},
        escape_block::EscapeBlock,
        expression::Expr,
        function::{Function, FunctionArg, FunctionCall},
        member_access::UnresolvedMemberAccess,
        module::Module,
        statement::{StmtBlock, VarDecl},
        structure::{Struct, StructAttr, StructInit, StructInitValue},
        tuple::{Tuple, TupleAccess},
        type_signature::TypeSignatureValue,
        IrAlloc, NodeRef,
    },
    IR,
};

#[derive(Debug)]
pub struct LowerAstResult<'a> {
    pub ctx: IrCtx<'a>,
    pub ir: IR<'a>,
}

pub fn lower_ast<'a>(ast: AST<'a>) -> LowerAstResult<'a> {
    let mut ctx = IrCtx::new();
    let module = ast.0;

    let stmt_block = ctx.lower_stmt(module.stmt);

    LowerAstResult {
        ctx,
        ir: IR(Module { stmt_block }),
    }
}

impl<'a> IrCtx<'a> {
    fn lower_stmt(
        &mut self,
        stmt: crate::ast::node::statement::Stmt<'a>,
    ) -> NodeRef<'a, StmtBlock<'a>> {
        fn unfold_stmts<'a>(
            ctx: &mut IrCtx<'a>,
            stmt: crate::ast::node::statement::Stmt<'a>,
            acc: &mut Vec<NodeRef<'a, Stmt<'a>>>,
        ) {
            match stmt.value {
                crate::ast::node::statement::StmtValue::VariableDecl(var_decl) => {
                    let var_decl_ref = VarDecl {
                        name: LateInit::empty(),
                        mutability: var_decl.mutability,
                        type_sig: LateInit::empty(),
                        value: ctx.lower_expr(var_decl.value),
                    }
                    .allocate(ctx);

                    ctx[var_decl_ref].name = ctx
                        .make_ident(var_decl.name, IdentParent::VarDeclName(var_decl_ref))
                        .into();

                    ctx[var_decl_ref].type_sig = var_decl
                        .type_sig
                        .map(|type_sig| {
                            type_sig
                                .into_ir_type(ctx, TypeSignatureParent::VarDeclSig(var_decl_ref))
                        })
                        .unwrap_or_else(|| {
                            ctx.make_type_var(TypeSignatureParent::VarDeclSig(var_decl_ref))
                        })
                        .into();

                    acc.push(Stmt::VariableDecl(var_decl_ref).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::FunctionDecl(func_decl) => {
                    let mut ir_args: Vec<NodeRef<'a, FunctionArg<'a>>> =
                        Vec::with_capacity(func_decl.args.len());
                    for arg in func_decl.args {
                        let func_arg = FunctionArg {
                            name: LateInit::empty(),
                            type_sig: LateInit::empty(),
                        }
                        .allocate(ctx);

                        ctx[func_arg].name = ctx
                            .make_ident(arg.name, IdentParent::FuncDeclArgName(func_arg))
                            .into();

                        ctx[func_arg].type_sig = arg
                            .type_sig
                            .map(|t| {
                                t.into_ir_type(ctx, TypeSignatureParent::FunctionDefArg(func_arg))
                            })
                            .unwrap_or_else(|| {
                                ctx.make_type_var(TypeSignatureParent::FunctionDefArg(func_arg))
                            })
                            .into();

                        ir_args.push(func_arg)
                    }

                    let body = ctx.lower_stmt(*func_decl.body);

                    let func = Function {
                        name: LateInit::empty(),
                        args: ir_args,
                        return_type: LateInit::empty(),
                        body,
                        span: func_decl.span,
                    }
                    .allocate(ctx);

                    let name = func_decl
                        .name
                        .map(|name| ctx.make_ident(name, IdentParent::FuncDeclName(func)))
                        .unwrap_or_else(|| ctx.make_anon_ident(IdentParent::FuncDeclName(func)));

                    ctx[func].name = name.into();

                    ctx[func].return_type = func_decl
                        .return_type
                        .map(|t| t.into_ir_type(ctx, TypeSignatureParent::FunctionDefReturn(func)))
                        .unwrap_or_else(|| {
                            ctx.make_type_var(TypeSignatureParent::FunctionDefReturn(func))
                        })
                        .into();

                    acc.push(Stmt::FunctionDecl(func).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::StructDecl(st_decl) => {
                    let mut ir_attrs: Vec<NodeRef<'a, StructAttr<'a>>> =
                        Vec::with_capacity(st_decl.attrs.len());
                    for attr in st_decl.attrs {
                        let st_attr = StructAttr {
                            name: LateInit::empty(),
                            mutability: attr.mutability,
                            type_sig: LateInit::empty(),
                            default_value: attr.default_value.map(|val| ctx.lower_expr(val)),
                        }
                        .allocate(ctx);

                        ctx[st_attr].name = ctx
                            .make_ident(attr.name, IdentParent::StructDeclAttrName(st_attr))
                            .into();

                        ctx[st_attr].type_sig = attr
                            .type_sig
                            .map(|t| t.into_ir_type(ctx, TypeSignatureParent::StructAttr(st_attr)))
                            .unwrap_or_else(|| {
                                ctx.make_type_var(TypeSignatureParent::StructAttr(st_attr))
                            })
                            .into();

                        ir_attrs.push(st_attr)
                    }

                    let st = Struct {
                        name: LateInit::empty(),
                        attrs: ir_attrs,
                    }
                    .allocate(ctx);

                    ctx[st].name = ctx
                        .make_ident(st_decl.name, IdentParent::StructDeclName(st))
                        .into();

                    acc.push(Stmt::StructDecl(st).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::EnumDecl(enm) => {
                    let mut values = Vec::with_capacity(enm.values.len());
                    for val in enm.values {
                        let enm_val = EnumValue {
                            name: LateInit::empty(),
                            items: LateInit::empty(),
                        }
                        .allocate(ctx);

                        ctx[enm_val].name = ctx
                            .make_ident(val.name, IdentParent::EnumDeclValueName(enm_val))
                            .into();

                        ctx[enm_val].items = val
                            .items
                            .into_iter()
                            .map(|t| t.into_ir_type(ctx, TypeSignatureParent::EnumValue(enm_val)))
                            .collect::<Vec<TypeSignature<'a>>>()
                            .into();

                        values.push(enm_val)
                    }

                    let enm_decl = Enum {
                        name: LateInit::empty(),
                        values,
                        type_sig: LateInit::empty(),
                    }
                    .allocate(ctx);

                    let enm_name_span = enm.name.span.clone();

                    let enm_name = ctx.make_ident(enm.name, IdentParent::EnumDeclName(enm_decl));
                    ctx[enm_decl].name = enm_name.into();
                    ctx[enm_decl].type_sig = ctx
                        .get_type_sig(
                            TypeSignatureValue::Enum { name: enm_name },
                            TypeSignatureContext {
                                parent: TypeSignatureParent::Enum(enm_decl),
                                type_span: Some(enm_name_span),
                            }
                            .alloc(),
                        )
                        .into();

                    acc.push(Stmt::EnumDecl(enm_decl).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::Compound(stmts) => {
                    for stmt in stmts {
                        unfold_stmts(ctx, stmt, acc);
                    }
                }
                crate::ast::node::statement::StmtValue::Expression(expr) => {
                    acc.push(Stmt::Expression(ctx.lower_expr(expr)).allocate(ctx));
                }
                crate::ast::node::statement::StmtValue::Return(expr) => {
                    acc.push(Stmt::Return(ctx.lower_expr(expr)).allocate(ctx));
                }
            };
        }

        let mut stmt_buffer = Vec::new();
        unfold_stmts(self, stmt, &mut stmt_buffer);

        StmtBlock(stmt_buffer).allocate(self)
    }

    fn lower_expr(
        &mut self,
        expr: crate::ast::node::expression::Expr<'a>,
    ) -> NodeRef<'a, Expr<'a>> {
        match expr.value {
            crate::ast::node::expression::ExprValue::StringLiteral(str) => {
                Expr::StringLiteral(str, expr.span).allocate(self)
            }
            crate::ast::node::expression::ExprValue::NumberLiteral(num) => {
                Expr::NumberLiteral(num, expr.span).allocate(self)
            }
            crate::ast::node::expression::ExprValue::BoolLiteral(bool) => {
                Expr::BoolLiteral(bool, expr.span).allocate(self)
            }
            crate::ast::node::expression::ExprValue::Function(func) => {
                let args = func
                    .args
                    .into_iter()
                    .map(|arg| {
                        let func_arg = FunctionArg {
                            name: LateInit::empty(),
                            type_sig: LateInit::empty(),
                        }
                        .allocate(self);

                        self[func_arg].name = self
                            .make_ident(arg.name, IdentParent::FuncDeclArgName(func_arg))
                            .into();

                        self[func_arg].type_sig = arg
                            .type_sig
                            .map(|t| {
                                t.into_ir_type(self, TypeSignatureParent::FunctionDefArg(func_arg))
                            })
                            .unwrap_or_else(|| {
                                self.make_type_var(TypeSignatureParent::FunctionDefArg(func_arg))
                            })
                            .into();

                        func_arg
                    })
                    .collect();

                let body = self.lower_stmt(*func.body);

                let func_decl = Function {
                    name: LateInit::empty(),
                    args,
                    return_type: LateInit::empty(),
                    body,
                    span: func.span,
                }
                .allocate(self);

                self[func_decl].name = func
                    .name
                    .map(|name| self.make_ident(name, IdentParent::FuncDeclName(func_decl)))
                    .unwrap_or_else(|| self.make_anon_ident(IdentParent::FuncDeclName(func_decl)))
                    .into();

                self[func_decl].return_type = func
                    .return_type
                    .map(|t| {
                        t.into_ir_type(self, TypeSignatureParent::FunctionDefReturn(func_decl))
                    })
                    .unwrap_or_else(|| {
                        self.make_type_var(TypeSignatureParent::FunctionDefReturn(func_decl))
                    })
                    .into();

                Expr::Function(func_decl).allocate(self)
            }
            crate::ast::node::expression::ExprValue::FunctionCall(func_call) => Expr::FunctionCall(
                FunctionCall {
                    func: self.lower_expr(func_call.func),
                    params: func_call
                        .params
                        .into_iter()
                        .map(|param| self.lower_expr(param))
                        .collect(),
                }
                .allocate(self),
            )
            .allocate(self),
            crate::ast::node::expression::ExprValue::Identifier(id) => {
                let id_expr = Expr::Identifier(LateInit::empty()).allocate(self);

                let unresolved_ident = self
                    .make_unresolved_ident(id, IdentParent::IdentExpr(id_expr).into())
                    .into();

                self[id_expr] = Expr::Identifier(unresolved_ident);

                id_expr
            }
            crate::ast::node::expression::ExprValue::StructInit(st_init) => {
                let struct_init = StructInit {
                    struct_name: LateInit::empty(),
                    scope_name: LateInit::empty(),
                    values: Vec::new(),
                }
                .allocate(self);

                self[struct_init].struct_name = self
                    .make_unresolved_ident(
                        st_init.struct_name,
                        IdentParent::StructInitStructName(struct_init).into(),
                    )
                    .into();

                self[struct_init].scope_name = self
                    .make_anon_ident(IdentParent::StructInitScopeName(struct_init))
                    .into();

                let st_init_vals = st_init
                    .values
                    .into_iter()
                    .map(|val| {
                        let st_val = StructInitValue {
                            name: LateInit::empty(),
                            parent: struct_init,
                            value: self.lower_expr(val.value),
                        }
                        .allocate(self);

                        self[st_val].name = self
                            .make_unresolved_ident(
                                val.name,
                                IdentParent::StructInitValueName(st_val).into(),
                            )
                            .into();

                        st_val
                    })
                    .collect();

                self[struct_init].values = st_init_vals;

                Expr::StructInit(struct_init).allocate(self)
            }
            crate::ast::node::expression::ExprValue::TupleAccess(tup_acc) => Expr::TupleAccess(
                TupleAccess {
                    tuple_expr: self.lower_expr(*tup_acc.tuple_expr),
                    attr: tup_acc.attr,
                }
                .allocate(self),
            )
            .allocate(self),
            crate::ast::node::expression::ExprValue::EscapeBlock(esc) => {
                let esc_blk = EscapeBlock {
                    content: esc.content,
                    type_sig: LateInit::empty(),
                }
                .allocate(self);

                self[esc_blk].type_sig = esc
                    .type_sig
                    .map(|t| t.into_ir_type(self, TypeSignatureParent::EscapeBlock(esc_blk)))
                    .unwrap_or_else(|| {
                        self.make_type_var(TypeSignatureParent::EscapeBlock(esc_blk))
                    })
                    .into();

                Expr::EscapeBlock(esc_blk).allocate(self)
            }
            crate::ast::node::expression::ExprValue::Assignment(asg) => Expr::Assignment(
                Assignment {
                    lhs: self.lower_expr(asg.lhs),
                    rhs: self.lower_expr(asg.rhs),
                }
                .allocate(self),
            )
            .allocate(self),
            crate::ast::node::expression::ExprValue::Tuple(tup) => {
                let tup_ref = Tuple {
                    values: tup
                        .values
                        .into_iter()
                        .map(|val| self.lower_expr(val))
                        .collect(),
                    type_sig: LateInit::empty(),
                }
                .allocate(self);

                self[tup_ref].type_sig = tup
                    .type_sig
                    .map(|t| t.into_ir_type(self, TypeSignatureParent::Tuple(tup_ref)))
                    .unwrap_or_else(|| self.make_type_var(TypeSignatureParent::Tuple(tup_ref)))
                    .into();

                Expr::Tuple(tup_ref)
            }
            .allocate(self),
            crate::ast::node::expression::ExprValue::MemberAccess(mem_acc) => {
                let object = mem_acc.object.map(|obj| self.lower_expr(obj));
                let items = mem_acc.items.map(|items| {
                    items
                        .into_iter()
                        .map(|item| self.lower_expr(item))
                        .collect()
                });

                let mem_acc_ref = UnresolvedMemberAccess {
                    object,
                    member_name: LateInit::empty(),
                    items,
                    type_sig: LateInit::empty(),
                }
                .allocate(self);

                self[mem_acc_ref].member_name = self
                    .make_unresolved_ident(
                        mem_acc.member_name,
                        IdentParent::MemberAccessMemberName(mem_acc_ref).into(),
                    )
                    .into();

                self[mem_acc_ref].type_sig = self
                    .make_type_var(TypeSignatureParent::MemberAccess(mem_acc_ref))
                    .into();

                Expr::UnresolvedMemberAccess(mem_acc_ref).allocate(self)
            }
        }
    }
}
