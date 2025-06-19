use crate::ast::{All, Any, Expr};
use crate::visit_mut::VisitMut;
use crate::visit_mut::walk_mut_expr_list;

fn contains_cross_same<T: Eq>(lhs: &[T], rhs: &[T]) -> bool {
    lhs.iter().any(|x| rhs.contains(x))
}

pub struct SimplifyNestedList;

impl<T> VisitMut<T> for SimplifyNestedList
where
    T: Eq,
{
    /// `any(x0, all(x0, x1), x2) => any(x0, x2)`
    fn visit_mut_any(&mut self, Any(any): &mut Any<T>) {
        walk_mut_expr_list(self, any);

        let mut i = 0;
        while i < any.len() {
            if let Expr::All(All(all)) = &any[i] {
                if contains_cross_same(all, any) {
                    any.remove(i);
                    continue;
                }
            }

            i += 1;
        }
    }

    /// `all(x0, any(x0, x1), x2) => all(x0, x2)`
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        walk_mut_expr_list(self, all);

        let mut i = 0;
        while i < all.len() {
            if let Expr::Any(Any(any)) = &all[i] {
                if contains_cross_same(any, all) {
                    all.remove(i);
                    continue;
                }
            }

            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, expr, not, var};

    #[test]
    fn simplify_any_with_nested_all_containing_same_element() {
        // any(x0, all(x0, x1), x2) => any(x0, x2)
        let mut x: Expr<u32> = expr(any((var(1), all((var(1), var(2))), var(3))));
        let expected: Expr<u32> = expr(any((var(1), var(3))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_all_with_nested_any_containing_same_element() {
        // all(x0, any(x0, x1), x2) => all(x0, x2)
        let mut x: Expr<u32> = expr(all((var(1), any((var(1), var(2))), var(3))));
        let expected: Expr<u32> = expr(all((var(1), var(3))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_simplify_any_with_nested_all_no_common_elements() {
        // any(x0, all(x1, x2), x3) should not be simplified
        let mut x: Expr<u32> = expr(any((var(1), all((var(2), var(4))), var(3))));
        let expected: Expr<u32> = expr(any((var(1), all((var(2), var(4))), var(3))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_simplify_all_with_nested_any_no_common_elements() {
        // all(x0, any(x1, x2), x3) should not be simplified
        let mut x: Expr<u32> = expr(all((var(1), any((var(2), var(4))), var(3))));
        let expected: Expr<u32> = expr(all((var(1), any((var(2), var(4))), var(3))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_multiple_nested_expressions_in_any() {
        // any(x0, all(x0, x1), all(x0, x2), x3) => any(x0, x3)
        let mut x: Expr<u32> = expr(any((
            var(1),
            all((var(1), var(2))),
            all((var(1), var(4))),
            var(3),
        )));
        let expected: Expr<u32> = expr(any((var(1), var(3))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_multiple_nested_expressions_in_all() {
        // all(x0, any(x0, x1), any(x0, x2), x3) => all(x0, x3)
        let mut x: Expr<u32> = expr(all((
            var(1),
            any((var(1), var(2))),
            any((var(1), var(4))),
            var(3),
        )));
        let expected: Expr<u32> = expr(all((var(1), var(3))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_nested_with_mixed_content() {
        // any(x0, all(x0, x1), not(x2)) => any(x0, not(x2))
        let mut x: Expr<u32> = expr(any((var(1), all((var(1), var(2))), not(var(3)))));
        let expected: Expr<u32> = expr(any((var(1), not(var(3)))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn simplify_deeply_nested_expressions() {
        // any(x0, all(x0, any(x1, x2))) => any(x0)
        let mut x: Expr<u32> = expr(any((var(1), all((var(1), any((var(2), var(3))))))));
        let expected: Expr<u32> = expr(any((var(1),)));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_change_when_no_simplification_possible() {
        // any(all(x0, x1), all(x2, x3)) should remain unchanged
        let mut x: Expr<u32> = expr(any((all((var(1), var(2))), all((var(3), var(4))))));
        let expected: Expr<u32> = expr(any((all((var(1), var(2))), all((var(3), var(4))))));

        SimplifyNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn contains_cross_same_function_test() {
        assert!(contains_cross_same(&[1, 2, 3], &[3, 4, 5]));
        assert!(contains_cross_same(&[1, 2], &[2, 3]));
        assert!(!contains_cross_same(&[1, 2], &[3, 4]));
        assert!(!contains_cross_same(&[], &[1, 2]));
        assert!(!contains_cross_same(&[1, 2], &[]));
    }
}
