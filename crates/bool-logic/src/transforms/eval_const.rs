use std::ops::Not as _;

use crate::ast::{All, Any, Expr, Not};
use crate::utils::remove_if;
use crate::visit_mut::VisitMut;
use crate::visit_mut::walk_mut_expr;

pub struct EvalConst;

impl EvalConst {
    fn eval_any<T>(any: &mut Vec<Expr<T>>) -> Option<bool> {
        remove_if(any, Expr::is_const_false);

        if any.is_empty() {
            return Some(false);
        }

        if any.iter().any(Expr::is_const_true) {
            return Some(true);
        }

        None
    }

    fn eval_all<T>(all: &mut Vec<Expr<T>>) -> Option<bool> {
        remove_if(all, Expr::is_const_true);

        if all.is_empty() {
            return Some(true);
        }

        if all.iter().any(Expr::is_const_false) {
            return Some(false);
        }

        None
    }

    fn eval_not<T>(not: &Expr<T>) -> Option<bool> {
        if let Expr::Const(val) = not {
            return Some(val.not());
        }
        None
    }
}

impl<T> VisitMut<T> for EvalConst {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_mut_expr(self, expr);

        match expr {
            Expr::Any(Any(any)) => {
                if let Some(val) = Self::eval_any(any) {
                    *expr = Expr::Const(val);
                }
            }
            Expr::All(All(all)) => {
                if let Some(val) = Self::eval_all(all) {
                    *expr = Expr::Const(val);
                }
            }
            Expr::Not(Not(not)) => {
                if let Some(val) = Self::eval_not(not) {
                    *expr = Expr::Const(val);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, const_, expr, not, var};

    #[test]
    fn eval_any_with_false() {
        let mut x: Expr<u32> = expr(any((const_(false), var(1))));
        let expected: Expr<u32> = expr(any((var(1),)));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_any_with_true() {
        let mut x: Expr<u32> = expr(any((var(1), const_(true), var(2))));
        let expected: Expr<u32> = expr(const_(true));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_any_all_false() {
        let mut x: Expr<u32> = expr(any((const_(false), const_(false))));
        let expected: Expr<u32> = expr(const_(false));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_all_with_true() {
        let mut x: Expr<u32> = expr(all((const_(true), var(1))));
        let expected: Expr<u32> = expr(all((var(1),)));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_all_with_false() {
        let mut x: Expr<u32> = expr(all((var(1), const_(false), var(2))));
        let expected: Expr<u32> = expr(const_(false));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_all_all_true() {
        let mut x: Expr<u32> = expr(all((const_(true), const_(true))));
        let expected: Expr<u32> = expr(const_(true));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_not_true() {
        let mut x: Expr<u32> = expr(not(const_(true)));
        let expected: Expr<u32> = expr(const_(false));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_not_false() {
        let mut x: Expr<u32> = expr(not(const_(false)));
        let expected: Expr<u32> = expr(const_(true));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_eval_not_variable() {
        let mut x: Expr<u32> = expr(not(var(1)));
        let expected: Expr<u32> = expr(not(var(1)));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_nested_expressions() {
        let mut x: Expr<u32> = expr(any((all((const_(true), var(1))), const_(false))));
        let expected: Expr<u32> = expr(any((all((var(1),)),)));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn eval_complex_nested() {
        let mut x: Expr<u32> = expr(not(all((const_(false), var(1)))));
        let expected: Expr<u32> = expr(const_(true));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_change_for_variables_only() {
        let mut x: Expr<u32> = expr(any((var(1), var(2))));
        let expected: Expr<u32> = expr(any((var(1), var(2))));

        EvalConst.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }
}
