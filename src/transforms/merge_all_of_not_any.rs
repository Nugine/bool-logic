use std::ops::Not as _;

use crate::ast::{All, Expr, Not, Var};
use crate::utils::{drain_filter, remove_if};
use crate::visit_mut::{VisitMut, walk_mut_expr_list};

fn as_mut_not_any<T>(expr: &mut Expr<T>) -> Option<&mut Vec<Expr<T>>> {
    expr.as_mut_not_any().map(|x| &mut x.0)
}

fn unwrap_expr_not_var<T>(expr: Expr<T>) -> Var<T> {
    if let Expr::Not(Not(not)) = expr {
        if let Expr::Var(var) = *not {
            return var;
        }
    }
    panic!()
}

pub struct MergeAllOfNotAny;

impl<T> VisitMut<T> for MergeAllOfNotAny {
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        walk_mut_expr_list(self, all);

        let mut not_any_list: Vec<_> = all.iter_mut().filter_map(as_mut_not_any).collect();

        if let [first, rest @ ..] = not_any_list.as_mut_slice() {
            if rest.is_empty().not() {
                for x in rest {
                    first.append(x);
                }
                remove_if(all, Expr::is_empty_not_any);
            }

            {
                let not_var_list: Vec<_> = drain_filter(all, |x| x.is_expr_not_var()).collect();
                let not_any = all.iter_mut().find_map(as_mut_not_any).unwrap();

                for not_var in not_var_list {
                    let var = unwrap_expr_not_var(not_var);
                    not_any.push(Expr::Var(var));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, expr, not, var};

    #[test]
    fn merge_multiple_not_any() {
        let mut x: Expr<u32> = expr(all((
            not(any((var(1), var(2)))),
            not(any((var(3), var(4)))),
            var(5),
        )));
        let expected: Expr<u32> = expr(all((not(any((var(1), var(2), var(3), var(4)))), var(5))));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn merge_not_var_into_not_any() {
        let mut x: Expr<u32> = expr(all((not(any((var(1), var(2)))), not(var(3)), var(4))));
        let expected: Expr<u32> = expr(all((not(any((var(1), var(2), var(3)))), var(4))));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn merge_multiple_not_any_and_not_var() {
        let mut x: Expr<u32> = expr(all((
            not(any((var(1),))),
            not(any((var(2),))),
            not(var(3)),
            not(var(4)),
            var(5),
        )));
        let expected: Expr<u32> = expr(all((not(any((var(1), var(2), var(3), var(4)))), var(5))));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn single_not_any_with_not_var() {
        let mut x: Expr<u32> = expr(all((not(any((var(1),))), not(var(2)), var(3))));
        let expected: Expr<u32> = expr(all((not(any((var(1), var(2)))), var(3))));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_merge_when_no_not_any() {
        let mut x: Expr<u32> = expr(all((var(1), var(2), not(var(3)))));
        let expected: Expr<u32> = expr(all((var(1), var(2), not(var(3)))));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_merge_when_single_not_any_no_not_var() {
        let mut x: Expr<u32> = expr(all((not(any((var(1), var(2)))), var(3))));
        let expected: Expr<u32> = expr(all((not(any((var(1), var(2)))), var(3))));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn merge_empty_not_any() {
        let mut x: Expr<u32> = expr(all((not(any((var(1),))), not(any(())), var(2))));
        let expected: Expr<u32> = expr(all((not(any((var(1),))), var(2))));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn complex_merge_scenario() {
        let mut x: Expr<u32> = expr(all((
            not(any((var(1), var(2)))),
            var(3),
            not(any((var(4),))),
            not(var(5)),
            not(any((var(6), var(7)))),
            not(var(8)),
        )));
        let expected: Expr<u32> = expr(all((
            not(any((
                var(1),
                var(2),
                var(4),
                var(6),
                var(7),
                var(5),
                var(8),
            ))),
            var(3),
        )));

        MergeAllOfNotAny.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }
}
