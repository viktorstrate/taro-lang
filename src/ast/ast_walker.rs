use std::fmt::Debug;

use super::{
    node::{
        identifier::Ident,
        module::Module,
        statement::{Stmt, VarDecl},
        structure::Struct,
    },
    AST,
};

#[allow(unused_variables)]
pub trait AstWalker<'a> {
    type Scope: Default = ();
    type Error: Debug = ();

    fn visit_var_decl(
        &mut self,
        scope: &mut Self::Scope,
        decl: &VarDecl<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_struct_decl(&mut self, st: &Struct<'a>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn visit_scope_begin(
        &mut self,
        parent: &mut Self::Scope,
        scope_ident: &Ident<'a>,
    ) -> Result<Self::Scope, Self::Error> {
        Ok(Self::Scope::default())
    }

    fn visit_scope_end(
        &mut self,
        parent: &mut Self::Scope,
        child: Self::Scope,
        scope_ident: &Ident<'a>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn walk_ast<'a, W: AstWalker<'a>>(walker: &mut W, ast: &AST<'a>) -> Result<W::Scope, W::Error> {
    let mut global_scope = W::Scope::default();
    walk_module(walker, &mut global_scope, &ast.0)?;
    Ok(global_scope)
}

fn walk_module<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    module: &Module<'a>,
) -> Result<(), W::Error> {
    for st in &module.structs {
        walk_struct(walker, scope, st)?;
    }

    for stmt in &module.stmts {
        walk_stmt(walker, scope, stmt)?;
    }

    Ok(())
}

fn walk_struct<'a, W: AstWalker<'a>>(
    walker: &mut W,
    _scope: &mut W::Scope,
    st: &Struct<'a>,
) -> Result<(), W::Error> {
    walker.visit_struct_decl(st)?;
    Ok(())
}

fn walk_stmt<'a, W: AstWalker<'a>>(
    walker: &mut W,
    scope: &mut W::Scope,
    stmt: &Stmt<'a>,
) -> Result<(), W::Error> {
    match stmt {
        Stmt::VarDecl(decl) => walker.visit_var_decl(scope, decl),
        Stmt::Compound(stmts) => {
            for stmt in stmts {
                walk_stmt(walker, scope, stmt)?;
            }
            Ok(())
        }
    }
}
