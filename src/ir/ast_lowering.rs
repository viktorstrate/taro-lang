use crate::{ast::AST, ir::node::statement::Stmt};

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
                    acc.push(
                        Stmt::VariableDecl(
                            VarDecl {
                                name: ctx.make_ident(var_decl.name),
                                mutability: var_decl.mutability,
                                type_sig: var_decl
                                    .type_sig
                                    .map(|type_sig| type_sig.into_ir_type(ctx))
                                    .unwrap_or_else(|| ctx.make_type_var()),
                                value: ctx.lower_expr(var_decl.value),
                            }
                            .allocate(ctx),
                        )
                        .allocate(ctx),
                    );
                }
                crate::ast::node::statement::StmtValue::FunctionDecl(func_decl) => {
                    let mut ir_args: Vec<NodeRef<'a, FunctionArg<'a>>> =
                        Vec::with_capacity(func_decl.args.len());
                    for arg in func_decl.args {
                        ir_args.push(
                            FunctionArg {
                                name: ctx.make_ident(arg.name),
                                type_sig: arg
                                    .type_sig
                                    .map(|t| t.into_ir_type(ctx))
                                    .unwrap_or_else(|| ctx.make_type_var()),
                            }
                            .allocate(ctx),
                        )
                    }

                    let name = func_decl
                        .name
                        .map(|name| ctx.make_ident(name))
                        .unwrap_or_else(|| ctx.make_anon_ident());

                    let return_type = func_decl
                        .return_type
                        .map(|t| t.into_ir_type(ctx))
                        .unwrap_or_else(|| ctx.make_type_var());

                    let body = ctx.lower_stmt(*func_decl.body);

                    acc.push(
                        Stmt::FunctionDecl(
                            Function {
                                name,
                                args: ir_args,
                                return_type,
                                body,
                            }
                            .allocate(ctx),
                        )
                        .allocate(ctx),
                    );
                }
                crate::ast::node::statement::StmtValue::StructDecl(st_decl) => {
                    let mut ir_attrs: Vec<NodeRef<'a, StructAttr<'a>>> =
                        Vec::with_capacity(st_decl.attrs.len());
                    for attr in st_decl.attrs {
                        ir_attrs.push(
                            StructAttr {
                                name: ctx.make_ident(attr.name),
                                mutability: attr.mutability,
                                type_sig: attr
                                    .type_sig
                                    .map(|t| t.into_ir_type(ctx))
                                    .unwrap_or_else(|| ctx.make_type_var()),
                                default_value: attr.default_value.map(|val| ctx.lower_expr(val)),
                            }
                            .allocate(ctx),
                        )
                    }
                    acc.push(
                        Stmt::StructDecl(
                            Struct {
                                name: ctx.make_ident(st_decl.name),
                                attrs: ir_attrs,
                            }
                            .allocate(ctx),
                        )
                        .allocate(ctx),
                    );
                }
                crate::ast::node::statement::StmtValue::EnumDecl(enm) => {
                    let mut values = Vec::with_capacity(enm.values.len());
                    for val in enm.values {
                        values.push(
                            EnumValue {
                                name: ctx.make_ident(val.name),
                                items: val.items.into_iter().map(|t| t.into_ir_type(ctx)).collect(),
                            }
                            .allocate(ctx),
                        )
                    }
                    let enm_name = ctx.make_ident(enm.name);
                    acc.push(
                        Stmt::EnumDecl(
                            Enum {
                                name: enm_name,
                                values,
                                type_sig: ctx
                                    .get_type_sig(TypeSignatureValue::Enum { name: enm_name }),
                            }
                            .allocate(ctx),
                        )
                        .allocate(ctx),
                    );
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
        let ir_expr: Expr<'a> = match expr.value {
            crate::ast::node::expression::ExprValue::StringLiteral(str) => Expr::StringLiteral(str),
            crate::ast::node::expression::ExprValue::NumberLiteral(num) => Expr::NumberLiteral(num),
            crate::ast::node::expression::ExprValue::BoolLiteral(bool) => Expr::BoolLiteral(bool),
            crate::ast::node::expression::ExprValue::Function(func) => {
                let name = func
                    .name
                    .map(|name| self.make_ident(name))
                    .unwrap_or_else(|| self.make_anon_ident());

                let args = func
                    .args
                    .into_iter()
                    .map(|arg| {
                        FunctionArg {
                            name: self.make_ident(arg.name),
                            type_sig: arg
                                .type_sig
                                .map(|t| t.into_ir_type(self))
                                .unwrap_or_else(|| self.make_type_var()),
                        }
                        .allocate(self)
                    })
                    .collect();

                let return_type = func
                    .return_type
                    .map(|t| t.into_ir_type(self))
                    .unwrap_or_else(|| self.make_type_var());

                let body = self.lower_stmt(*func.body);

                Expr::Function(
                    Function {
                        name,
                        args,
                        return_type,
                        body,
                    }
                    .allocate(self),
                )
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
            ),
            crate::ast::node::expression::ExprValue::Identifier(id) => {
                Expr::Identifier(self.make_unresolved_ident(id))
            }
            crate::ast::node::expression::ExprValue::StructInit(st_init) => {
                let struct_init = StructInit {
                    struct_name: self.make_unresolved_ident(st_init.struct_name),
                    scope_name: self.make_anon_ident(),
                    values: Vec::new(),
                }
                .allocate(self);

                let st_init_vals = st_init
                    .values
                    .into_iter()
                    .map(|val| {
                        StructInitValue {
                            name: self.make_unresolved_ident(val.name),
                            parent: struct_init,
                            value: self.lower_expr(val.value),
                        }
                        .allocate(self)
                    })
                    .collect();

                self[struct_init].values = st_init_vals;

                Expr::StructInit(struct_init)
            }
            crate::ast::node::expression::ExprValue::TupleAccess(tup_acc) => Expr::TupleAccess(
                TupleAccess {
                    tuple_expr: self.lower_expr(*tup_acc.tuple_expr),
                    attr: tup_acc.attr,
                }
                .allocate(self),
            ),
            crate::ast::node::expression::ExprValue::EscapeBlock(esc) => Expr::EscapeBlock(
                EscapeBlock {
                    content: esc.content,
                    type_sig: esc
                        .type_sig
                        .map(|t| t.into_ir_type(self))
                        .unwrap_or_else(|| self.make_type_var()),
                }
                .allocate(self),
            ),
            crate::ast::node::expression::ExprValue::Assignment(asg) => Expr::Assignment(
                Assignment {
                    lhs: self.lower_expr(asg.lhs),
                    rhs: self.lower_expr(asg.rhs),
                }
                .allocate(self),
            ),
            crate::ast::node::expression::ExprValue::Tuple(tup) => Expr::Tuple(
                Tuple {
                    values: tup
                        .values
                        .into_iter()
                        .map(|val| self.lower_expr(val))
                        .collect(),
                    type_sig: tup
                        .type_sig
                        .map(|t| t.into_ir_type(self))
                        .unwrap_or_else(|| self.make_type_var()),
                }
                .allocate(self),
            ),
            crate::ast::node::expression::ExprValue::MemberAccess(mem_acc) => {
                let object = mem_acc.object.map(|obj| self.lower_expr(obj));
                let items = mem_acc
                    .items
                    .into_iter()
                    .map(|item| self.lower_expr(item))
                    .collect();
                Expr::UnresolvedMemberAccess(
                    UnresolvedMemberAccess {
                        object,
                        member_name: self.make_unresolved_ident(mem_acc.member_name),
                        items,
                    }
                    .allocate(self),
                )
            }
        };

        ir_expr.allocate(self)
    }
}
