use id_arena::Id;

use crate::{
    ir::context::IrCtx,
    symbols::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolValueItem},
};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    type_signature::{Mutability, TypeEvalError, TypeSignature, TypeSignatureValue, Typed},
    NodeRef,
};

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: Ident<'a>,
    pub attrs: Vec<NodeRef<'a, StructAttr<'a>>>,
}

#[derive(Debug)]
pub struct StructAttr<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub default_value: Option<NodeRef<'a, Expr<'a>>>,
}

#[derive(Debug)]
pub struct StructInit<'a> {
    pub struct_name: Ident<'a>,
    pub scope_name: Ident<'a>,
    pub values: Vec<NodeRef<'a, StructInitValue<'a>>>,
}

#[derive(Debug)]
pub struct StructInitValue<'a> {
    pub name: Ident<'a>,
    pub value: NodeRef<'a, Expr<'a>>,
}

#[derive(Debug)]
pub struct StructAccess<'a> {
    pub struct_expr: NodeRef<'a, Expr<'a>>,
    pub attr_name: Ident<'a>,
}

impl<'a> NodeRef<'a, Struct<'a>> {
    pub fn lookup_attr(
        &self,
        ident: Ident<'a>,
        ctx: &IrCtx<'a>,
    ) -> Option<NodeRef<'a, StructAttr<'a>>> {
        ctx[*self]
            .attrs
            .iter()
            .find(|attr| ctx[**attr].name == ident)
            .map(|attr| *attr)
    }
}

impl<'a> Identifiable<'a> for Struct<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

impl<'a> Identifiable<'a> for StructAttr<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.name
    }
}

impl<'a> Identifiable<'a> for StructInit<'a> {
    fn name(&self, _ctx: &IrCtx<'a>) -> Ident<'a> {
        self.scope_name
    }
}

impl<'a> Typed<'a> for NodeRef<'a, Struct<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let name = ctx[*self].name;
        Ok(ctx.get_type_sig(TypeSignatureValue::Struct { name }))
    }
}

impl<'a> Typed<'a> for NodeRef<'a, StructAttr<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match &ctx[*self].default_value {
            Some(value) => value.eval_type(symbols, ctx),
            None => {
                let type_sig = ctx[*self]
                    .type_sig
                    .clone()
                    .expect("struct should have at least a type signature or a default value");

                // TODO: Resolve in symbol_resolver
                let type_sig = if let TypeSignatureValue::Unresolved(type_ident) = ctx[type_sig] {
                    symbols
                        .lookup(ctx, type_ident)
                        .ok_or(TypeEvalError::UnknownIdentifier(type_ident))?
                        .clone()
                        .eval_type(symbols, ctx)?
                } else {
                    type_sig
                };

                Ok(type_sig)
            }
        }
    }

    fn specified_type(&self, ctx: &mut IrCtx<'a>) -> Option<TypeSignature<'a>> {
        ctx[*self].type_sig
    }

    fn specify_type(
        &mut self,
        ctx: &mut IrCtx<'a>,
        new_type: TypeSignature<'a>,
    ) -> Result<(), TypeEvalError<'a>> {
        ctx[*self].type_sig = Some(new_type);
        Ok(())
    }
}

impl<'a> NodeRef<'a, StructInit<'a>> {
    pub fn lookup_struct(
        &self,
        ctx: &IrCtx<'a>,
        symbols: &SymbolTableZipper<'a>,
    ) -> Option<NodeRef<'a, Struct<'a>>> {
        let sym_val = symbols.lookup(ctx, ctx[*self].struct_name);

        match sym_val.map(|val| &ctx[*val]) {
            Some(SymbolValueItem::StructDecl(st)) => Some(*st),
            _ => None,
        }
    }
}

impl<'a> Typed<'a> for NodeRef<'a, StructInit<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let st = self
            .lookup_struct(ctx, symbols)
            .ok_or(TypeEvalError::UnknownIdentifier(ctx[*self].struct_name))?;

        Ok(ctx.get_type_sig(TypeSignatureValue::Struct { name: ctx[st].name }))
    }
}

impl<'a> Typed<'a> for NodeRef<'a, StructAccess<'a>> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
        ctx: &mut IrCtx<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        self.lookup_attr(ctx, symbols)?.eval_type(symbols, ctx)
    }
}

impl<'a> NodeRef<'a, StructAccess<'a>> {
    pub fn lookup_attr(
        &self,
        ctx: &mut IrCtx<'a>,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<NodeRef<'a, StructAttr<'a>>, TypeEvalError<'a>> {
        let st_type = ctx[*self].struct_expr.eval_type(symbols, ctx)?;
        let struct_name = match &ctx[st_type] {
            TypeSignatureValue::Struct { name } => *name,
            _ => return Err(TypeEvalError::AccessNonStruct(st_type)),
        };

        let st_sym = symbols
            .lookup(ctx, struct_name)
            .ok_or(TypeEvalError::UnknownIdentifier(struct_name))?;

        let st = match ctx[*st_sym] {
            SymbolValueItem::StructDecl(st) => st,
            _ => unreachable!("symbol type should match up with expr eval"),
        };

        let attr_name = ctx[*self].attr_name;
        st.lookup_attr(attr_name, ctx)
            .ok_or(TypeEvalError::UnknownIdentifier(attr_name))
    }

    pub fn lookup_attr_chain<'c>(
        &self,
        ctx: &mut IrCtx<'a>,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<Vec<NodeRef<'a, StructAttr<'a>>>, TypeEvalError<'a>> {
        fn recursive_lookup<'a>(
            result: &mut Vec<NodeRef<'a, StructAttr<'a>>>,
            st_access: NodeRef<'a, StructAccess<'a>>,
            ctx: &mut IrCtx<'a>,
            symbols: &mut SymbolTableZipper<'a>,
        ) -> Result<(), TypeEvalError<'a>> {
            let inner_expr = ctx[st_access].struct_expr;
            if let Expr::StructAccess(inner_access) = &ctx[inner_expr] {
                recursive_lookup(result, *inner_access, ctx, symbols)?;
            }

            let attr = st_access.lookup_attr(ctx, symbols)?;
            result.push(attr);

            Ok(())
        }

        let mut result = Vec::new();
        recursive_lookup(&mut result, *self, ctx, symbols)?;

        Ok(result)
    }
}

// #[cfg(test)]
// mod tests {
//     use std::assert_matches::assert_matches;

//     use crate::{ir::test_utils::utils::type_check, parser::parse_ast};

//     #[test]
//     fn test_nested_struct() {
//         let mut ast = parse_ast(
//             "
//         struct Deep {
//             let mut inner = false
//         }

//         struct Foo {
//             let mut bar: Deep
//         }

//         let foo = Foo { bar: Deep {} }
//         foo.bar.inner = true
//         ",
//         )
//         .unwrap();
//         assert_matches!(type_check(&mut ast), Ok(()))
//     }
// }