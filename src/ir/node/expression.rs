use id_arena::Id;



use super::{
    assignment::Assignment,
    escape_block::EscapeBlock,
    function::{Function, FunctionCall},
    identifier::Ident,
    structure::{StructAccess, StructInit},
    tuple::{Tuple, TupleAccess},
};

#[derive(Debug)]
pub enum Expr<'a> {
    StringLiteral(&'a str),
    NumberLiteral(f64),
    BoolLiteral(bool),
    Function(Id<Function<'a>>),
    FunctionCall(Id<FunctionCall<'a>>),
    Identifier(Ident<'a>),
    StructInit(Id<StructInit<'a>>),
    StructAccess(Id<StructAccess<'a>>),
    TupleAccess(Id<TupleAccess<'a>>),
    EscapeBlock(Id<EscapeBlock<'a>>),
    Assignment(Id<Assignment<'a>>),
    Tuple(Id<Tuple<'a>>),
}

// impl<'a> Typed<'a> for Expr<'a> {
//     fn eval_type(
//         &self,
//         symbols: &mut SymbolTableZipper<'a>,
//         ctx: &mut IrCtx<'a>,
//     ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
//         match self {
//             Expr::StringLiteral(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::String)),
//             Expr::NumberLiteral(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Number)),
//             Expr::BoolLiteral(_) => Ok(ctx.get_builtin_type_sig(BuiltinType::Boolean)),
//             Expr::Function(func) => ctx.nodes.funcs[*func].eval_type(symbols, ctx),
//             Expr::FunctionCall(call) => ctx.nodes.func_calls[*call].eval_type(symbols, ctx),
//             Expr::Identifier(ident) => {
//                 let sym_val = symbols
//                     .lookup(ctx, *ident)
//                     .ok_or(TypeEvalError::UnknownIdentifier(*ident))?;

//                 ctx.symbols[*sym_val].eval_type(symbol, ctxs)
//             }
//             Expr::StructInit(struct_init) => {
//                 ctx.nodes.st_inits[*struct_init].eval_type(symbols, ctx)
//             }
//             Expr::StructAccess(struct_access) => {
//                 ctx.nodes.st_accs[*struct_access].eval_type(symbol, ctxs)
//             }
//             Expr::EscapeBlock(block) => ctx.nodes.esc_blks[*block].eval_type(symbols, ctx),
//             Expr::Assignment(asg) => {
//                 ctx.nodes.exprs[ctx.nodes.asgns[*asg].rhs].eval_type(symbols, ctx)
//             }
//             Expr::Tuple(tup) => ctx.nodes.tups[*tup].eval_type(symbols, ctx),
//             Expr::TupleAccess(tup_acc) => ctx.nodes.tup_accs[*tup_acc].eval_type(symbols, ctx),
//         }
//     }

//     fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
//         match self {
//             Expr::StringLiteral(_) => None,
//             Expr::NumberLiteral(_) => None,
//             Expr::BoolLiteral(_) => None,
//             Expr::Function(func) => ctx.nodes.funcs[*func].specified_type(ctx),
//             Expr::FunctionCall(call) => ctx.nodes.func_calls[*call].specified_type(ctx),
//             Expr::Identifier(_) => None,
//             Expr::StructInit(st_init) => ctx.nodes.st_inits[*st_init].specified_type(ctx),
//             Expr::StructAccess(_) => None,
//             Expr::EscapeBlock(block) => ctx.nodes.esc_blks[*block].specified_type(ctx),
//             Expr::Assignment(_) => None,
//             Expr::Tuple(tup) => ctx.nodes.tups[*tup].specified_type(ctx),
//             Expr::TupleAccess(tup_acc) => ctx.nodes.tup_accs[*tup_acc].specified_type(ctx),
//         }
//     }

//     fn specify_type(
//         &mut self,
//         ctx: &mut IrCtx<'a>,
//         new_type: TypeSignature<'a>,
//     ) -> Result<(), TypeEvalError<'a>> {
//         match self {
//             Expr::StringLiteral(_) => Ok(()),
//             Expr::NumberLiteral(_) => Ok(()),
//             Expr::BoolLiteral(_) => Ok(()),
//             Expr::Function(func) => ctx.nodes.funcs[*func].specify_type(new_type, ctx),
//             Expr::FunctionCall(call) => ctx.nodes.func_calls[*call].specify_type(new_type, ctx),
//             Expr::Identifier(_) => Ok(()),
//             Expr::StructInit(st_init) => ctx.nodes.st_inits[*st_init].specify_type(new_type, ctx),
//             Expr::StructAccess(_) => Ok(()),
//             Expr::EscapeBlock(block) => ctx.nodes.esc_blks[*block].specify_type(new_type, ctx),
//             Expr::Assignment(_) => Ok(()),
//             Expr::Tuple(tup) => ctx.nodes.tups[*tup].specify_type(new_type, ctx),
//             Expr::TupleAccess(tup_acc) => ctx.nodes.tup_accs[*tup_acc].specify_type(new_type, ctx),
//         }
//     }
// }
