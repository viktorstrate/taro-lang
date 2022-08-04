use crate::ast::AST;

use super::{
    context::{IrAllocate, IrCtx},
    node::{
        assignment::Assignment,
        enumeration::{Enum, EnumValue},
        escape_block::EscapeBlock,
        expression::Expr,
        function::{Function, FunctionArg, FunctionCall},
        identifier::Ident,
        module::Module,
        statement::{Stmt, VarDecl},
        structure::{Struct, StructAccess, StructAttr, StructInit, StructInitValue},
        tuple::{Tuple, TupleAccess},
    },
    IR,
};

impl<'a, 'ctx> IrCtx<'a, 'ctx> {
    pub fn lower_ast(&'ctx self, ast: AST<'a>) -> IR<'a, 'ctx> {
        let module = ast.0;

        let stmts: Vec<&'ctx mut Stmt<'a, 'ctx>> = module
            .stmts
            .into_iter()
            .map(|stmt| self.lower_stmt(stmt))
            .collect();

        IR(Module { stmts })
    }

    fn lower_stmt(
        &'ctx self,
        stmt: crate::ast::node::statement::Stmt<'a>,
    ) -> &'ctx mut Stmt<'a, 'ctx> {
        let ir_stmt = match stmt.value {
            crate::ast::node::statement::StmtValue::VariableDecl(var_decl) => {
                Stmt::VariableDecl(VarDecl {
                    name: self.make_ident(var_decl.name),
                    mutability: var_decl.mutability,
                    type_sig: var_decl
                        .type_sig
                        .map(|type_sig| self.get_type_sig(type_sig)),
                    value: self.lower_expr(var_decl.value),
                })
            }
            crate::ast::node::statement::StmtValue::FunctionDecl(func_decl) => {
                let mut ir_args: Vec<&mut FunctionArg<'a, 'ctx>> =
                    Vec::with_capacity(func_decl.args.len());
                for arg in func_decl.args {
                    ir_args.push(
                        FunctionArg {
                            name: self.make_ident(arg.name),
                            type_sig: arg.type_sig.map(|t| self.get_type_sig(t)),
                        }
                        .allocate(self),
                    )
                }
                Stmt::FunctionDecl(Function {
                    name: func_decl
                        .name
                        .map(|name| self.make_ident(name))
                        .unwrap_or_else(|| self.make_anon_ident()),
                    args: ir_args,
                    return_type: func_decl.return_type.map(|t| self.get_type_sig(t)),
                    body: self.lower_stmt(*func_decl.body),
                })
            }
            crate::ast::node::statement::StmtValue::StructDecl(st_decl) => {
                let mut ir_attrs: Vec<&mut StructAttr<'a, 'ctx>> =
                    Vec::with_capacity(st_decl.attrs.len());
                for attr in st_decl.attrs {
                    ir_attrs.push(
                        StructAttr {
                            name: self.make_ident(attr.name),
                            mutability: attr.mutability,
                            type_sig: attr.type_sig.map(|t| self.get_type_sig(t)),
                            default_value: attr.default_value.map(|val| self.lower_expr(val)),
                        }
                        .allocate(self),
                    )
                }
                Stmt::StructDecl(Struct {
                    name: self.make_ident(st_decl.name),
                    attrs: ir_attrs,
                })
            }
            crate::ast::node::statement::StmtValue::EnumDecl(enm) => {
                let mut values = Vec::with_capacity(enm.values.len());
                for val in enm.values {
                    values.push(
                        EnumValue {
                            name: self.make_ident(val.name),
                            items: val
                                .items
                                .into_iter()
                                .map(|t| self.get_type_sig(t))
                                .collect(),
                        }
                        .allocate(self),
                    )
                }
                Stmt::EnumDecl(Enum {
                    name: self.make_ident(enm.name),
                    values,
                })
            }
            crate::ast::node::statement::StmtValue::Compound(stmts) => Stmt::Compound(
                stmts
                    .into_iter()
                    .map(|stmt| self.lower_stmt(stmt))
                    .collect(),
            ),
            crate::ast::node::statement::StmtValue::Expression(expr) => {
                Stmt::Expression(self.lower_expr(expr))
            }
            crate::ast::node::statement::StmtValue::Return(expr) => {
                Stmt::Return(self.lower_expr(expr))
            }
        };

        ir_stmt.allocate(self)
    }

    fn lower_expr(
        &'ctx self,
        expr: crate::ast::node::expression::Expr<'a>,
    ) -> &'ctx mut Expr<'a, 'ctx> {
        let _ir_expr: Expr<'a, 'ctx> = match expr.value {
            crate::ast::node::expression::ExprValue::StringLiteral(str) => Expr::StringLiteral(str),
            crate::ast::node::expression::ExprValue::NumberLiteral(num) => Expr::NumberLiteral(num),
            crate::ast::node::expression::ExprValue::BoolLiteral(bool) => Expr::BoolLiteral(bool),
            crate::ast::node::expression::ExprValue::Function(func) => Expr::Function(
                Function {
                    name: func
                        .name
                        .map(|name| self.make_ident(name))
                        .unwrap_or_else(|| self.make_anon_ident()),
                    args: func
                        .args
                        .into_iter()
                        .map(|arg| {
                            FunctionArg {
                                name: self.make_ident(arg.name),
                                type_sig: arg.type_sig.map(|t| self.get_type_sig(t)),
                            }
                            .allocate(self)
                        })
                        .collect(),
                    return_type: func.return_type.map(|t| self.get_type_sig(t)),
                    body: self.lower_stmt(*func.body),
                }
                .allocate(self),
            ),
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
            crate::ast::node::expression::ExprValue::StructInit(st_init) => Expr::StructInit(
                StructInit {
                    struct_name: self.make_unresolved_ident(st_init.struct_name),
                    scope_name: self.make_anon_ident(),
                    values: st_init
                        .values
                        .into_iter()
                        .map(|val| {
                            StructInitValue {
                                name: self.make_ident(val.name),
                                value: self.lower_expr(val.value),
                            }
                            .allocate(self)
                        })
                        .collect(),
                }
                .allocate(self),
            ),
            crate::ast::node::expression::ExprValue::StructAccess(st_acc) => Expr::StructAccess(
                StructAccess {
                    struct_expr: self.lower_expr(*st_acc.struct_expr),
                    attr_name: self.make_ident(st_acc.attr_name),
                }
                .allocate(self),
            ),
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
                    type_sig: esc.type_sig.map(|t| self.get_type_sig(t)),
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
                    type_sig: tup.type_sig.map(|t| self.get_type_sig(t)),
                }
                .allocate(self),
            ),
        };

        todo!()
    }
}
