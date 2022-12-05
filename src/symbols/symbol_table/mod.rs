use std::collections::{HashMap, VecDeque};

use id_arena::Id;

use crate::ir::{
    context::{IrArenaType, IrCtx},
    node::{
        control_flow::{IfBranchBody, IfStmt},
        enumeration::{Enum, EnumValue},
        external::ExternalObject,
        function::{Function, FunctionArg},
        identifier::{Ident, IdentKey, Identifiable, ResolvedIdentValue},
        statement::VarDecl,
        structure::{Struct, StructAttr, StructInit},
        traits::Trait,
        type_signature::{TypeEvalError, TypeSignature, Typed},
        NodeRef,
    },
};

use self::symbol_table_zipper::SymbolTableZipper;

pub mod symbol_table_zipper;

#[derive(Debug)]
pub enum SymbolCollectionError<'a> {
    SymbolAlreadyExistsInScope {
        new: Ident<'a>,
        existing: SymbolValue<'a>,
    },
    ScopeNotFound(Ident<'a>),
}

#[derive(Default, Debug)]
pub struct SymbolTable<'a> {
    /// Symbols that are available from the entire scope
    pub(super) scope_global_table: HashMap<IdentKey<'a>, SymbolValue<'a>>,
    /// An list for order dependent symbols such as variable declarations,
    /// where symbols are first available after they have been declared.
    pub(super) ordered_symbols: VecDeque<SymbolValue<'a>>,
    /// Nested scopes such as function bodies and struct definitions.
    pub(super) scopes: HashMap<IdentKey<'a>, SymbolTable<'a>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// A value returned from a symbol lookup
pub struct SymbolValue<'a> {
    id: Id<SymbolValueItem<'a>>,
}

impl<'a> SymbolValue<'a> {
    pub fn unwrap_struct(&self, ctx: &IrCtx<'a>) -> NodeRef<'a, Struct<'a>> {
        match ctx[*self] {
            SymbolValueItem::StructDecl(st) => st,
            sym => panic!("Expected symbol to be struct: {sym:?}"),
        }
    }

    pub fn unwrap_enum(&self, ctx: &IrCtx<'a>) -> NodeRef<'a, Enum<'a>> {
        match ctx[*self] {
            SymbolValueItem::EnumDecl(enm) => enm,
            sym => panic!("Expected symbol to be enum: {sym:?}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SymbolValueItem<'a> {
    BuiltinType(Ident<'a>),
    VarDecl(NodeRef<'a, VarDecl<'a>>),
    FuncDecl(NodeRef<'a, Function<'a>>),
    FuncArg(NodeRef<'a, FunctionArg<'a>>),
    StructDecl(NodeRef<'a, Struct<'a>>),
    StructAttr(NodeRef<'a, StructAttr<'a>>),
    StructInit(NodeRef<'a, StructInit<'a>>),
    EnumDecl(NodeRef<'a, Enum<'a>>),
    EnumValue(NodeRef<'a, EnumValue<'a>>),
    ExternalObject(NodeRef<'a, ExternalObject<'a>>),
    IfBranch(NodeRef<'a, IfStmt<'a>>, IfBranchBody),
    TraitDecl(NodeRef<'a, Trait<'a>>),
}

impl<'a> Into<Id<SymbolValueItem<'a>>> for SymbolValue<'a> {
    fn into(self) -> Id<SymbolValueItem<'a>> {
        self.id
    }
}

impl<'a> From<Id<SymbolValueItem<'a>>> for SymbolValue<'a> {
    fn from(id: Id<SymbolValueItem<'a>>) -> Self {
        Self { id }
    }
}

impl<'a> From<NodeRef<'a, Function<'a>>> for SymbolValueItem<'a> {
    fn from(func: NodeRef<'a, Function<'a>>) -> Self {
        Self::FuncDecl(func)
    }
}

impl<'a> From<NodeRef<'a, VarDecl<'a>>> for SymbolValueItem<'a> {
    fn from(var: NodeRef<'a, VarDecl<'a>>) -> Self {
        Self::VarDecl(var)
    }
}

impl<'a> Identifiable<'a> for SymbolValueItem<'a> {
    fn name(&self, ctx: &IrCtx<'a>) -> Ident<'a> {
        match self {
            SymbolValueItem::BuiltinType(builtin) => *builtin,
            SymbolValueItem::VarDecl(var) => ctx[*var].name(ctx),
            SymbolValueItem::FuncDecl(func) => ctx[*func].name(ctx),
            SymbolValueItem::FuncArg(arg) => ctx[*arg].name(ctx),
            SymbolValueItem::StructDecl(st) => ctx[*st].name(ctx),
            SymbolValueItem::StructAttr(attr) => ctx[*attr].name(ctx),
            SymbolValueItem::StructInit(st_init) => ctx[*st_init].name(ctx),
            SymbolValueItem::EnumDecl(enm) => ctx[*enm].name(ctx),
            SymbolValueItem::EnumValue(enm_val) => ctx[*enm_val].name(ctx),
            SymbolValueItem::ExternalObject(obj) => ctx[*obj].name(ctx),
            SymbolValueItem::IfBranch(ifb, branch) => ctx[*ifb].branch_ident(*branch),
            SymbolValueItem::TraitDecl(tr) => ctx[*tr].name(ctx),
        }
    }
}

impl<'a> SymbolValue<'a> {
    pub fn describe_type(&self, ctx: &IrCtx<'a>) -> &'static str {
        match ctx[*self] {
            SymbolValueItem::BuiltinType(_) => "builtin type",
            SymbolValueItem::VarDecl(_) => "variable",
            SymbolValueItem::FuncDecl(_) => "function",
            SymbolValueItem::FuncArg(_) => "function argument",
            SymbolValueItem::StructDecl(_) => "struct declaration",
            SymbolValueItem::StructAttr(_) => "struct attribute",
            SymbolValueItem::StructInit(_) => "struct",
            SymbolValueItem::EnumDecl(_) => "enum",
            SymbolValueItem::EnumValue(_) => "enum value",
            SymbolValueItem::ExternalObject(_) => "external object",
            SymbolValueItem::IfBranch(_, _) => "if branch",
            SymbolValueItem::TraitDecl(_) => "trait",
        }
    }
}

impl<'a> IrArenaType<'a> for SymbolValueItem<'a> {
    fn arena<'b>(ctx: &'b IrCtx<'a>) -> &'b id_arena::Arena<Self> {
        &ctx.symbols
    }

    fn arena_mut<'b>(ctx: &'b mut IrCtx<'a>) -> &'b mut id_arena::Arena<Self> {
        &mut ctx.symbols
    }
}

impl SymbolValueItem<'_> {
    fn is_order_dependent(&self) -> bool {
        match self {
            SymbolValueItem::VarDecl(_) => true,
            _ => false,
        }
    }
}

impl<'a> Typed<'a> for SymbolValue<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match ctx[*self].clone() {
            SymbolValueItem::BuiltinType(builtin) => match &ctx[builtin] {
                crate::ir::node::identifier::IdentValue::Resolved(
                    ResolvedIdentValue::BuiltinType(builtin_type),
                ) => Ok(ctx.get_builtin_type_sig(*builtin_type)),
                _ => unreachable!("ident should resolve to builtin type"),
            },
            SymbolValueItem::VarDecl(var) => var.eval_type(symbols, ctx),
            SymbolValueItem::FuncDecl(decl) => decl.eval_type(symbols, ctx),
            SymbolValueItem::FuncArg(arg) => arg.eval_type(symbols, ctx),
            SymbolValueItem::StructDecl(st) => st.eval_type(symbols, ctx),
            SymbolValueItem::StructAttr(attr) => attr.eval_type(symbols, ctx),
            SymbolValueItem::StructInit(st_init) => st_init.eval_type(symbols, ctx),
            SymbolValueItem::EnumDecl(enm) => enm.eval_type(symbols, ctx),
            SymbolValueItem::EnumValue(enm_val) => enm_val.eval_type(symbols, ctx),
            SymbolValueItem::ExternalObject(obj) => obj.eval_type(symbols, ctx),
            SymbolValueItem::IfBranch(_, _) => unreachable!(),
            SymbolValueItem::TraitDecl(_) => todo!(),
        }
    }

    fn specified_type(&self, ctx: &IrCtx<'a>) -> Option<TypeSignature<'a>> {
        match ctx[*self].clone() {
            SymbolValueItem::BuiltinType(_) => None,
            SymbolValueItem::VarDecl(var) => var.specified_type(ctx),
            SymbolValueItem::FuncDecl(decl) => decl.specified_type(ctx),
            SymbolValueItem::FuncArg(arg) => arg.specified_type(ctx),
            SymbolValueItem::StructDecl(st) => st.specified_type(ctx),
            SymbolValueItem::StructAttr(attr) => attr.specified_type(ctx),
            SymbolValueItem::StructInit(st_init) => st_init.specified_type(ctx),
            SymbolValueItem::EnumDecl(enm) => enm.specified_type(ctx),
            SymbolValueItem::EnumValue(enm_val) => enm_val.specified_type(ctx),
            SymbolValueItem::ExternalObject(obj) => obj.specified_type(ctx),
            SymbolValueItem::IfBranch(_, _) => unreachable!(),
            SymbolValueItem::TraitDecl(_) => todo!(),
        }
    }

    fn specify_type(
        &self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        match ctx[*self].clone() {
            SymbolValueItem::BuiltinType(_) => Ok(()),
            SymbolValueItem::VarDecl(var) => var.specify_type(ctx, new_type),
            SymbolValueItem::FuncDecl(decl) => decl.specify_type(ctx, new_type),
            SymbolValueItem::FuncArg(arg) => arg.specify_type(ctx, new_type),
            SymbolValueItem::StructDecl(st) => st.specify_type(ctx, new_type),
            SymbolValueItem::StructAttr(attr) => attr.specify_type(ctx, new_type),
            SymbolValueItem::StructInit(st_init) => st_init.specify_type(ctx, new_type),
            SymbolValueItem::EnumDecl(enm) => enm.specify_type(ctx, new_type),
            SymbolValueItem::EnumValue(enm_val) => enm_val.specify_type(ctx, new_type),
            SymbolValueItem::ExternalObject(obj) => obj.specify_type(ctx, new_type),
            SymbolValueItem::IfBranch(_, _) => unreachable!(),
            SymbolValueItem::TraitDecl(_) => todo!(),
        }
    }
}

impl<'a> SymbolTable<'a> {
    pub fn insert(
        &mut self,
        ctx: &mut IrCtx<'a>,
        val: SymbolValueItem<'a>,
    ) -> Result<SymbolValue<'a>, SymbolCollectionError<'a>> {
        let new_sym = ctx.make_symbol(val);
        let val = &ctx[new_sym];
        if val.is_order_dependent() {
            self.ordered_symbols.push_back(new_sym);
        } else {
            let ident = val.name(ctx);
            let key = IdentKey::from_ident(ctx, ident);
            self.scope_global_table
                .try_insert(key, new_sym)
                .map_err(
                    move |err| SymbolCollectionError::SymbolAlreadyExistsInScope {
                        new: ident,
                        existing: err.entry.get().clone(),
                    },
                )?;
        }
        Ok(new_sym)
    }

    pub fn insert_scope(
        &mut self,
        ctx: &IrCtx<'a>,
        ident: Ident<'a>,
        scope: SymbolTable<'a>,
    ) -> Result<&mut SymbolTable<'a>, SymbolCollectionError<'a>> {
        match self
            .scopes
            .try_insert(IdentKey::from_ident(ctx, ident), scope)
        {
            Ok(scp) => Ok(scp),
            Err(_) => {
                let scope_val = self
                    .scope_global_table
                    .get(&IdentKey::from_ident(ctx, ident))
                    .cloned()
                    .expect("scope should have associated symbol value");

                Err(SymbolCollectionError::SymbolAlreadyExistsInScope {
                    new: ident,
                    existing: scope_val,
                })
            }
        }
    }

    pub fn remove_scope(&mut self, ctx: &IrCtx<'a>, ident: Ident<'a>) -> Option<SymbolTable<'a>> {
        self.scopes.remove(&IdentKey::from_ident(ctx, ident))
    }

    pub fn lookup_global_table(
        &self,
        ctx: &IrCtx<'a>,
        ident: Ident<'a>,
    ) -> Option<&SymbolValue<'a>> {
        self.scope_global_table
            .get(&IdentKey::from_ident(ctx, ident))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::ir::{
        node::expression::Expr,
        test_utils::utils::{collect_symbols, lowered_ir},
    };

    use super::*;

    #[test]
    fn test_locate_ordered_symbol() {
        let mut ir = lowered_ir("let x: Boolean = true").unwrap();
        let symtable = collect_symbols(&mut ir).unwrap();

        let sym_val = *symtable.ordered_symbols.front().unwrap();

        assert_eq!(
            IdentKey::from_ident(&ir.ctx, ir.ctx[sym_val].name(&ir.ctx)),
            IdentKey::Named("x")
        );

        match ir.ctx[sym_val] {
            SymbolValueItem::VarDecl(var_decl) => {
                assert_matches!(ir.ctx[ir.ctx[var_decl].value], Expr::BoolLiteral(true, _));
            }
            _ => assert!(false),
        }
    }
}
