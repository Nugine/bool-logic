#![allow(clippy::single_match)]

use crate::ast::{All, Any, Expr, Not};
use crate::visit_mut::VisitMut;
use crate::visit_mut::walk_mut_expr;

fn take<T>(expr: &mut Expr<T>) -> Expr<T> {
    std::mem::replace(expr, Expr::Const(false))
}

pub struct FlattenSingle;

impl<T> VisitMut<T> for FlattenSingle {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_mut_expr(self, expr);

        match expr {
            Expr::Any(Any(any)) => match any.as_mut_slice() {
                [] => *expr = Expr::Const(false),
                [sub] => *expr = take(sub),
                _ => {}
            },
            Expr::All(All(all)) => match all.as_mut_slice() {
                [] => *expr = Expr::Const(true),
                [sub] => *expr = take(sub),
                _ => {}
            },
            Expr::Not(Not(not)) => match &mut **not {
                Expr::Not(Not(sub)) => *expr = take(sub),
                _ => {}
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, const_, expr, not, var};

    #[test]
    fn flatten_empty_any() {
        let mut x: Expr<u32> = expr(any(()));
        let expected: Expr<u32> = expr(const_(false));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_empty_all() {
        let mut x: Expr<u32> = expr(all(()));
        let expected: Expr<u32> = expr(const_(true));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_single_any() {
        let mut x: Expr<u32> = expr(any((var(1),)));
        let expected: Expr<u32> = expr(var(1));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_single_all() {
        let mut x: Expr<u32> = expr(all((var(1),)));
        let expected: Expr<u32> = expr(var(1));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_double_negation() {
        let mut x: Expr<u32> = expr(not(not(var(1))));
        let expected: Expr<u32> = expr(var(1));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_flatten_multiple_any() {
        let mut x: Expr<u32> = expr(any((var(1), var(2))));
        let expected: Expr<u32> = expr(any((var(1), var(2))));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_flatten_multiple_all() {
        let mut x: Expr<u32> = expr(all((var(1), var(2))));
        let expected: Expr<u32> = expr(all((var(1), var(2))));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_flatten_single_negation() {
        let mut x: Expr<u32> = expr(not(var(1)));
        let expected: Expr<u32> = expr(not(var(1)));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_nested_expressions() {
        let mut x: Expr<u32> = expr(any((all((var(1),)),)));
        let expected: Expr<u32> = expr(var(1));

        FlattenSingle.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }
}
