use nom::{
    bytes::complete::tag,
    combinator::{map, opt},
    sequence::{preceded, tuple},
};

use crate::ast::node::control_flow::IfStmt;

use super::{
    expression::expression, spaced, span, statement::statement, surround_brackets, BracketType,
    Input, Res,
};

pub fn if_branch(i: Input<'_>) -> Res<Input<'_>, IfStmt<'_>> {
    // if EXPR "{" STMT "}" [ else "{" STMT "}" ]

    map(
        span(tuple((
            preceded(spaced(tag("if")), expression),
            surround_brackets(BracketType::Curly, statement),
            opt(preceded(
                spaced(tag("else")),
                surround_brackets(BracketType::Curly, statement),
            )),
        ))),
        |(span, (expr, stmt, else_stmt))| IfStmt {
            condition: expr,
            span,
            body: Box::new(stmt),
            else_body: else_stmt.map(Box::new),
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{
        ast::node::{
            expression::{Expr, ExprValue},
            statement::{Stmt, StmtValue},
        },
        parser::new_input,
    };

    use super::*;

    #[test]
    fn test_if_branch() {
        let if_br = if_branch(new_input("if true { return 1 } else { return 0 }"))
            .unwrap()
            .1;
        assert_matches!(
            if_br.condition,
            Expr {
                span: _,
                value: ExprValue::BoolLiteral(true)
            }
        );
        assert_matches!(
            *if_br.body,
            Stmt {
                span: _,
                value: StmtValue::Return(_)
            }
        );
        assert!(if_br.else_body.is_some());
    }
}
