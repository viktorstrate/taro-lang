use crate::{
    ast::ref_generator::RefID,
    symbols::symbol_table::{symbol_table_zipper::SymbolTableZipper, SymbolValue},
};

use super::{
    expression::Expr,
    identifier::{Ident, Identifiable},
    type_signature::{Mutability, TypeEvalError, TypeSignature, Typed},
};

#[derive(Debug, Clone)]
pub struct Struct<'a> {
    pub name: Ident<'a>,
    pub attrs: Vec<StructAttr<'a>>,
    pub ref_id: RefID,
}

#[derive(Debug, Clone)]
pub struct StructAttr<'a> {
    pub name: Ident<'a>,
    pub mutability: Mutability,
    pub type_sig: Option<TypeSignature<'a>>,
    pub default_value: Option<Expr<'a>>,
}

#[derive(Debug, Clone)]
pub struct StructInit<'a> {
    pub name: Ident<'a>,
    pub values: Vec<StructInitValue<'a>>,
}

#[derive(Debug, Clone)]
pub struct StructInitValue<'a> {
    pub name: Ident<'a>,
    pub value: Expr<'a>,
}

#[derive(Debug, Clone)]
pub struct StructAccess<'a> {
    pub struct_expr: Box<Expr<'a>>,
    pub attr_name: Ident<'a>,
}

impl<'a> Struct<'a> {
    fn lookup_attr(&self, ident: &Ident<'a>) -> Option<&StructAttr<'a>> {
        self.attrs.iter().find(|attr| *attr.name() == *ident)
    }
}

impl<'a> Identifiable<'a> for Struct<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Identifiable<'a> for StructAttr<'a> {
    fn name(&self) -> &Ident<'a> {
        &self.name
    }
}

impl<'a> Typed<'a> for Struct<'a> {
    fn eval_type(
        &self,
        _symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        Ok(TypeSignature::Struct {
            name: self.name.clone(),
            ref_id: self.ref_id,
        })
    }
}

impl<'a> Typed<'a> for StructAttr<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        match &self.default_value {
            Some(value) => value.eval_type(symbols),
            None => {
                let type_sig = self
                    .type_sig
                    .clone()
                    .expect("struct should have at least a type signature or a default value");

                let type_sig = if let TypeSignature::Base(type_ident) = type_sig {
                    symbols
                        .lookup(&type_ident)
                        .ok_or(TypeEvalError::UnknownIdentifier(type_ident))?
                        .clone()
                        .eval_type(symbols)?
                } else {
                    type_sig
                };

                Ok(type_sig)
            }
        }
    }

    fn specified_type(&self) -> Option<TypeSignature<'a>> {
        self.type_sig.clone()
    }

    fn specify_type(&mut self, new_type: TypeSignature<'a>) {
        self.type_sig = Some(new_type);
    }
}

impl<'a> StructInit<'a> {
    pub fn lookup_struct<'b>(&self, symbols: &'b SymbolTableZipper<'a>) -> Option<&'b Struct<'a>> {
        let sym_val = symbols.lookup(&self.name);

        match sym_val {
            Some(SymbolValue::StructDecl(st)) => Some(st),
            _ => None,
        }
    }
}

impl<'a> Typed<'a> for StructInit<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        let st = self
            .lookup_struct(symbols)
            .ok_or(TypeEvalError::UnknownIdentifier(self.name.clone()))?;

        Ok(TypeSignature::Struct {
            name: st.name.clone(),
            ref_id: st.ref_id,
        })
    }
}

impl<'a> Typed<'a> for StructAccess<'a> {
    fn eval_type(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<TypeSignature<'a>, TypeEvalError<'a>> {
        self.lookup_attr(symbols)?.clone().eval_type(symbols)
    }
}

impl<'a> StructAccess<'a> {
    pub fn lookup_attr<'b>(
        &self,
        symbols: &'b mut SymbolTableZipper<'a>,
    ) -> Result<&'b StructAttr<'a>, TypeEvalError<'a>> {
        let struct_name = match self.struct_expr.eval_type(symbols)? {
            TypeSignature::Struct { name, ref_id: _ } => name,
            val => return Err(TypeEvalError::AccessNonStruct(val)),
        };

        let st_sym = symbols
            .lookup(&struct_name)
            .ok_or(TypeEvalError::UnknownIdentifier(struct_name))?;

        let st = match st_sym {
            SymbolValue::StructDecl(st) => st,
            _ => unreachable!("symbol type should match up with expr eval"),
        };

        st.lookup_attr(&self.attr_name)
            .ok_or(TypeEvalError::UnknownIdentifier(self.attr_name.clone()))
    }

    pub fn lookup_attr_chain<'c>(
        &self,
        symbols: &mut SymbolTableZipper<'a>,
    ) -> Result<Vec<StructAttr<'a>>, TypeEvalError<'a>> {
        fn recursive_lookup<'a>(
            result: &mut Vec<StructAttr<'a>>,
            st_access: &StructAccess<'a>,
            symbols: &mut SymbolTableZipper<'a>,
        ) -> Result<(), TypeEvalError<'a>> {
            if let Expr::StructAccess(inner_access) = st_access.struct_expr.as_ref() {
                recursive_lookup(result, inner_access, symbols)?;
            }

            let attr = st_access.lookup_attr(symbols)?;
            result.push(attr.clone());

            Ok(())
        }

        let mut result = Vec::new();
        recursive_lookup(&mut result, self, symbols)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{ast::test_utils::utils::type_check, parser::parse_ast};

    #[test]
    fn test_nested_struct() {
        let mut ast = parse_ast(
            "
        struct Deep {
            let mut inner = false
        }

        struct Foo {
            let mut bar: Deep
        }

        let foo = Foo { bar: Deep {} }
        foo.bar.inner = true
        ",
        )
        .unwrap();
        assert_matches!(type_check(&mut ast), Ok(()))
    }
}
