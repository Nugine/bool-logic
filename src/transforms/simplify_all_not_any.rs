use std::slice;

use crate::ast::{All, Any, Expr, Not};
use crate::utils::remove_if;
use crate::visit_mut::VisitMut;
use crate::visit_mut::walk_mut_expr_list;

pub struct SimplifyAllNotAny;

impl<T> VisitMut<T> for SimplifyAllNotAny
where
    T: Eq,
{
    /// `all(not(any(x0, x1, ...)), any(x0, x2, ...)) => all(not(any(...)), any(x0, ...))`
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        walk_mut_expr_list(self, all);

        match all.as_mut_slice() {
            [Expr::Any(Any(pos)), Expr::Not(Not(not))]
            | [Expr::Not(Not(not)), Expr::Any(Any(pos))] => {
                let neg = match not.as_mut_any() {
                    Some(Any(neg)) => neg,
                    None => slice::from_mut(&mut **not),
                };
                remove_if(pos, |x| neg.contains(x));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, expr, not, var};

    #[test]
    fn simplify_all_not_any_basic() {
        let mut x = expr(all((not(any((var(1), var(2)))), any((var(1), var(3))))));
        let expected = expr(all((not(any((var(1), var(2)))), any((var(3),)))));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_all_not_any_reverse_order() {
        let mut x = expr(all((any((var(1), var(3))), not(any((var(1), var(2)))))));
        let expected = expr(all((any((var(3),)), not(any((var(1), var(2)))))));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_all_not_any_complete_overlap() {
        let mut x = expr(all((not(any((var(1), var(2)))), any((var(1), var(2))))));
        let expected = expr(all((not(any((var(1), var(2)))), any(()))));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_all_not_any_no_overlap() {
        let mut x = expr(all((not(any((var(1), var(2)))), any((var(3), var(4))))));
        let expected = expr(all((not(any((var(1), var(2)))), any((var(3), var(4))))));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_all_not_expr_direct() {
        let mut x = expr(all((not(var(1)), any((var(1), var(2))))));
        let expected = expr(all((not(var(1)), any((var(2),)))));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_simplify_wrong_pattern() {
        let mut x = expr(all((var(1), var(2), var(3))));
        let expected = expr(all((var(1), var(2), var(3))));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_simplify_single_expr() {
        let mut x = expr(all((not(any((var(1), var(2)))),)));
        let expected = expr(all((not(any((var(1), var(2)))),)));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_partial_overlap() {
        let mut x = expr(all((
            not(any((var(1), var(2), var(3)))),
            any((var(1), var(4), var(5))),
        )));
        let expected = expr(all((
            not(any((var(1), var(2), var(3)))),
            any((var(4), var(5))),
        )));

        SimplifyAllNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }
}
