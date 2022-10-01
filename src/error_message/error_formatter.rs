use std::{collections::VecDeque, io::Write};

use crate::{ir::context::IrCtx, parser::Span};

pub trait Spanned<'a> {
    fn get_span(&self, ctx: &IrCtx<'a>) -> Option<Span<'a>>;
}

#[derive(Debug, Default)]
pub struct SpanMsg<'a> {
    pub msg: Option<&'a str>,
    pub msg_type: ErrMsgType,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrMsgType {
    Text,
    Warn,
    Err,
    Note,
    Hint,
}

impl Default for ErrMsgType {
    fn default() -> Self {
        Self::Text
    }
}

#[derive(Debug, PartialOrd, Ord)]
pub struct SpanItem<'a> {
    pub span: Span<'a>,
    pub msg: Option<String>,
    pub err_type: ErrMsgType,
}

impl<'a> Eq for SpanItem<'a> {}

impl<'a> PartialEq for SpanItem<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.span.line == other.span.line
            && self.span.offset == other.span.offset
            && self.span.fragment == other.span.fragment
            && self.msg == other.msg
            && self.err_type == other.err_type
    }
}

pub struct ErrRemark {
    pub msg: String,
    pub err_type: ErrMsgType,
}

pub fn format_span_items<'a>(
    w: &mut impl Write,
    items: &mut [SpanItem<'a>],
    remarks: &[ErrRemark],
) -> Result<(), std::io::Error> {
    items.sort();

    if !items.is_empty() {
        let mut lines = items[0].span.source.lines();
        let mut current_line = 0;
        let mut current_col = 0;
        let mut line = "";
        let mut line_msgs = VecDeque::new();

        let max_lines = items[0].span.source.chars().filter(|c| *c == '\n').count();
        let line_decimals = max_lines.to_string().chars().count();

        for (i, item) in items.iter().enumerate() {
            if item.span.line > current_line {
                let offset = item.span.line - current_line - 1;
                lines.advance_by(offset).unwrap();
                current_line += offset + 1;
                current_col = 0;
                line = lines.next().unwrap();
            }

            if !item.span.fragment.contains("\n") {
                if current_col == 0 {
                    let line_padding =
                        " ".repeat(line_decimals - item.span.line.to_string().chars().count());
                    writeln!(w, "{}{} | {}", line_padding, item.span.line, line)?;
                }

                let start_offset = line_decimals + 2 + item.span.offset - current_col;
                let span_len = item.span.fragment.len();

                debug_assert!(start_offset > 0);

                write!(w, "{}{}", " ".repeat(start_offset), "^".repeat(span_len))?;
                current_col += start_offset + span_len;

                if let Some(msg) = item.msg.as_ref() {
                    line_msgs.push_back((msg, start_offset));
                }
            } else {
                dbg!(&item);
                todo!()
            }

            if item == items.last().unwrap() || items[i + 1].span.line > item.span.line {
                if line_msgs.is_empty() {
                    write!(w, "\n\n")?;
                }

                for (i, (current_msg, _)) in line_msgs.iter().enumerate().rev() {
                    writeln!(w, " {}", current_msg)?;

                    for n in 0..2 {
                        for (_, line_offset) in line_msgs.iter().take(i) {
                            write!(w, "{}|", " ".repeat(*line_offset))?;
                        }
                        if n == 0 {
                            writeln!(w)?;
                        }
                    }
                }
                line_msgs.clear();
            }
        }
    }

    for remark in remarks {
        let prefix = match remark.err_type {
            ErrMsgType::Text => "",
            ErrMsgType::Warn => "warn: ",
            ErrMsgType::Err => "error: ",
            ErrMsgType::Note => "note: ",
            ErrMsgType::Hint => "hint: ",
        };

        writeln!(w, "{}{}\n", prefix, remark.msg)?;
    }

    Ok(())
}
