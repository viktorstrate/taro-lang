use crate::{
    parser::{ParserError, Span},
    symbols::{symbol_resolver::SymbolResolutionError, symbol_table::SymbolCollectionError},
    type_checker::TypeCheckerError,
    TranspilerError,
};
use std::io::Write;

pub mod error_formatter;
pub mod error_msg_utils;
pub mod sym_collect_errors;
pub mod sym_resolution_errors;
pub mod type_check_errors;
pub mod type_eval_errors;

pub struct ErrMsg<'a, 'ret, W: Write> {
    pub span: Option<Span<'a>>,
    pub title: Box<dyn Fn(&mut W) -> Result<(), std::io::Error> + 'ret>,
    pub msg: Box<dyn Fn(&mut W) -> Result<(), std::io::Error> + 'ret>,
}

pub trait ErrorMessage<'a: 'ret, 'ret, T, W>
where
    W: Write,
{
    fn err_msg(&'ret self, ctx: T) -> ErrMsg<'a, 'ret, W>;

    fn format_err(&'ret self, w: &mut W, ctx: T) -> Result<(), std::io::Error> {
        let err_msg = self.err_msg(ctx);

        write!(w, "error: ")?;
        (*err_msg.title)(w)?;
        writeln!(w)?;

        if let Some(span) = err_msg.span {
            writeln!(w, "/path/to/file.taro:{}:{}\n", span.line, span.offset)?;
        }

        (*err_msg.msg)(w)?;

        writeln!(w)?;

        w.flush()
    }
}

impl<'a: 'ret, 'ret, W: Write> ErrorMessage<'a, 'ret, (), W> for ParserError<'a> {
    fn err_msg(&self, _ctx: ()) -> ErrMsg<'a, 'ret, W> {
        let err_msg = self.to_string();

        ErrMsg {
            span: None,
            title: Box::new(|w| write!(w, "Parser error")),
            msg: Box::new(move |w| writeln!(w, "\n{}", err_msg)),
        }
    }
}

impl<'a: 'ret, 'ret, W: Write> ErrorMessage<'a, 'ret, (), W> for TranspilerError<'a> {
    fn err_msg(&'ret self, _ctx: ()) -> ErrMsg<'a, 'ret, W> {
        match self {
            TranspilerError::Parse(err) => ParserError::err_msg(err, ()),
            TranspilerError::SymbolCollectError(la, err) => {
                SymbolCollectionError::err_msg(err, &la.ctx)
            }
            TranspilerError::SymbolResolveError(la, err) => {
                SymbolResolutionError::err_msg(err, &la.ctx)
            }
            TranspilerError::TypeCheck(type_check, la, err) => {
                TypeCheckerError::err_msg(err, (type_check, &la.ctx))
            }
            TranspilerError::Write(err) => {
                let err_msg = err.to_string();
                ErrMsg {
                    span: None,
                    title: Box::new(|w| write!(w, "IO write error")),
                    msg: Box::new(move |w| writeln!(w, "{}", err_msg)),
                }
            }
        }
    }
}
