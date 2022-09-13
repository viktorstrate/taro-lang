use crate::{
    ir::{
        context::IrCtx,
        node::identifier::{Ident, IdentValue, ResolvedIdentValue},
    },
    parser::{ParserError, Span},
    symbols::{symbol_resolver::SymbolResolutionError, symbol_table::SymbolCollectionError},
    type_checker::TypeCheckerError,
    TranspilerError,
};
use std::io::Write;

pub mod sym_collect_errors;

impl<'a> Ident<'a> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        match &ctx[*self] {
            IdentValue::Resolved(id) => match id {
                ResolvedIdentValue::Named { def_span, name: _ } => Some(def_span.clone()),
                ResolvedIdentValue::Anonymous => None,
                ResolvedIdentValue::BuiltinType(_) => None,
            },
            IdentValue::Unresolved(id) => Some(id.span.clone()),
        }
    }
}

impl<'a> Span<'a> {
    fn format_spanned_code(&self, w: &mut impl Write, msg: &str) -> Result<(), std::io::Error> {
        let mut lines = self.source.lines();

        if self.line > 0 {
            lines.advance_by(self.line - 1).unwrap();
        }

        let line = lines.next().unwrap();

        if !self.fragment.contains("\n") {
            writeln!(w, "{} | {}", self.line, line)?;
            writeln!(
                w,
                "{}{} - {}",
                " ".repeat(3 + self.offset),
                "^".repeat(self.fragment.len()),
                msg
            )?;
        } else {
            todo!()
        }

        Ok(())
    }
}

pub trait ErrorMessage<'a, T, W>
where
    T: Copy,
    W: Write,
{
    fn err_span(&self, ctx: T) -> Option<Span<'a>>;
    fn err_title(&self, w: &mut W, ctx: T) -> Result<(), std::io::Error>;
    fn err_msg(&self, w: &mut W, ctx: T) -> Result<(), std::io::Error>;

    fn format_err(&self, w: &mut W, ctx: T) -> Result<(), std::io::Error> {
        write!(w, "error: ")?;
        self.err_title(w, ctx)?;
        writeln!(w)?;

        if let Some(span) = self.err_span(ctx) {
            writeln!(w, "/path/to/file.taro:{}:{}\n", span.line, span.offset)?;
        }

        self.err_msg(w, ctx)?;
        w.flush()
    }
}

impl<'a, W: Write> ErrorMessage<'a, (), W> for ParserError<'a> {
    fn err_span(&self, _: ()) -> Option<Span<'a>> {
        todo!()
    }

    fn err_title(&self, _w: &mut W, _: ()) -> Result<(), std::io::Error> {
        todo!()
    }

    fn err_msg(&self, _w: &mut W, _: ()) -> Result<(), std::io::Error> {
        todo!()
    }
}

impl<'a, W: Write> ErrorMessage<'a, &IrCtx<'a>, W> for SymbolResolutionError<'a> {
    fn err_span(&self, _ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        todo!()
    }

    fn err_title(&self, _w: &mut W, _ctx: &IrCtx<'a>) -> Result<(), std::io::Error> {
        todo!()
    }

    fn err_msg(&self, _w: &mut W, _ctx: &IrCtx<'a>) -> Result<(), std::io::Error> {
        todo!()
    }
}

impl<'a, W: Write> ErrorMessage<'a, &IrCtx<'a>, W> for TypeCheckerError<'a> {
    fn err_span(&self, _ctx: &IrCtx<'a>) -> Option<Span<'a>> {
        todo!()
    }

    fn err_title(&self, _w: &mut W, _ctx: &IrCtx<'a>) -> Result<(), std::io::Error> {
        todo!()
    }

    fn err_msg(&self, _w: &mut W, _ctx: &IrCtx<'a>) -> Result<(), std::io::Error> {
        todo!()
    }
}

impl<'a, W: Write> ErrorMessage<'a, (), W> for TranspilerError<'a> {
    fn err_span(&self, _ctx: ()) -> Option<Span<'a>> {
        match self {
            TranspilerError::Parse(err) => {
                <ParserError<'a> as ErrorMessage<'a, (), W>>::err_span(err, ())
            }
            TranspilerError::SymbolCollectError(la, err) => {
                <SymbolCollectionError<'a> as ErrorMessage<'a, &IrCtx<'a>, W>>::err_span(
                    err, &la.ctx,
                )
            }
            TranspilerError::SymbolResolveError(la, err) => {
                <SymbolResolutionError<'a> as ErrorMessage<'a, &IrCtx<'a>, W>>::err_span(
                    err, &la.ctx,
                )
            }
            TranspilerError::TypeCheck(la, err) => {
                <TypeCheckerError<'a> as ErrorMessage<'a, &IrCtx<'a>, W>>::err_span(err, &la.ctx)
            }
            TranspilerError::Write(_) => None,
        }
    }

    fn err_title(&self, w: &mut W, ctx: ()) -> Result<(), std::io::Error> {
        match self {
            TranspilerError::Parse(err) => err.err_title(w, ctx),
            TranspilerError::SymbolCollectError(la, err) => err.err_title(w, &la.ctx),
            TranspilerError::SymbolResolveError(la, err) => err.err_title(w, &la.ctx),
            TranspilerError::TypeCheck(la, err) => err.err_title(w, &la.ctx),
            TranspilerError::Write(_) => write!(w, "IO write error"),
        }
    }

    fn err_msg(&self, w: &mut W, ctx: ()) -> Result<(), std::io::Error> {
        match self {
            TranspilerError::Parse(err) => err.err_msg(w, ctx),
            TranspilerError::SymbolCollectError(la, err) => err.err_msg(w, &la.ctx),
            TranspilerError::SymbolResolveError(la, err) => err.err_msg(w, &la.ctx),
            TranspilerError::TypeCheck(la, err) => err.err_msg(w, &la.ctx),
            TranspilerError::Write(err) => write!(w, "{}", err.to_string()),
        }
    }
}
