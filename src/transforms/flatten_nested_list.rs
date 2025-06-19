use crate::ast::{All, Any, Expr};
use crate::visit_mut::VisitMut;
use crate::visit_mut::walk_mut_expr_list;

pub struct FlattenNestedList;

impl<T> VisitMut<T> for FlattenNestedList {
    fn visit_mut_any(&mut self, Any(list): &mut Any<T>) {
        walk_mut_expr_list(self, list);

        if list.iter().any(Expr::is_any) {
            let mut ans: Vec<Expr<T>> = Vec::with_capacity(list.len());
            for item in list.drain(..) {
                if let Expr::Any(Any(any)) = item {
                    ans.extend(any);
                } else {
                    ans.push(item);
                }
            }
            *list = ans;
        }
    }

    fn visit_mut_all(&mut self, All(list): &mut All<T>) {
        walk_mut_expr_list(self, list);

        if list.iter().any(Expr::is_all) {
            let mut ans: Vec<Expr<T>> = Vec::with_capacity(list.len());
            for item in list.drain(..) {
                if let Expr::All(All(all)) = item {
                    ans.extend(all);
                } else {
                    ans.push(item);
                }
            }
            *list = ans;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, const_, expr, not, var};

    #[test]
    fn flatten_nested_any() {
        let mut x: Expr<u32> = expr(any((any((var(1), var(2))), any((var(3), var(4))))));
        let expected: Expr<u32> = expr(any((var(1), var(2), var(3), var(4))));

        FlattenNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_nested_all() {
        let mut x: Expr<u32> = expr(all((all((var(1), var(2))), all((var(3), var(4))))));
        let expected: Expr<u32> = expr(all((var(1), var(2), var(3), var(4))));

        FlattenNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_mixed_nested() {
        let mut x: Expr<u32> = expr(any((any((var(1), var(2))), var(3), any((var(4), var(5))))));
        let expected: Expr<u32> = expr(any((var(1), var(2), var(3), var(4), var(5))));

        FlattenNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_deep_nested() {
        let mut x: Expr<u32> = expr(any((
            any((any((var(1), var(2))), var(3))),
            any((var(4), var(5))),
        )));
        let expected: Expr<u32> = expr(any((var(1), var(2), var(3), var(4), var(5))));

        FlattenNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_flatten_mixed_types() {
        let mut x: Expr<u32> = expr(any((all((var(1), var(2))), var(3))));
        let expected: Expr<u32> = expr(any((all((var(1), var(2))), var(3))));

        FlattenNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_single_nested() {
        let mut x: Expr<u32> = expr(any(any((var(1), var(2)))));
        let expected: Expr<u32> = expr(any((var(1), var(2))));

        FlattenNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn flatten_complex_expression() {
        let mut x: Expr<u32> = expr(all((
            all((var(1), not(var(2)))),
            all((const_(true), var(3))),
            var(4),
        )));
        let expected: Expr<u32> = expr(all((var(1), not(var(2)), const_(true), var(3), var(4))));

        FlattenNestedList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }
}
