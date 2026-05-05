use crate::ast::Expr;
use crate::visit_mut::VisitMut;
use crate::visit_mut::walk_mut_expr;

pub struct DedupList;

impl<T> VisitMut<T> for DedupList
where
    T: Eq,
{
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_mut_expr(self, expr);

        if let Some(list) = expr.as_mut_expr_list() {
            let mut i = 0;
            while i < list.len() {
                let mut j = i + 1;
                while j < list.len() {
                    if list[i] == list[j] {
                        list.remove(j);
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, const_, expr, not, var};

    #[test]
    fn dedup_any_with_duplicates() {
        let mut x: Expr<u32> = expr(any((var(1), var(2), var(1), var(3), var(2))));
        let expected: Expr<u32> = expr(any((var(1), var(2), var(3))));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_all_with_duplicates() {
        let mut x: Expr<u32> = expr(all((var(1), var(2), var(1), var(3), var(2))));
        let expected: Expr<u32> = expr(all((var(1), var(2), var(3))));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_no_duplicates() {
        let mut x: Expr<u32> = expr(any((var(1), var(2), var(3))));
        let expected: Expr<u32> = expr(any((var(1), var(2), var(3))));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_empty_list() {
        let mut x: Expr<u32> = expr(any(()));
        let expected: Expr<u32> = expr(any(()));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_single_element() {
        let mut x: Expr<u32> = expr(any((var(1),)));
        let expected: Expr<u32> = expr(any((var(1),)));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_all_same_elements() {
        let mut x: Expr<u32> = expr(any((var(1), var(1), var(1), var(1))));
        let expected: Expr<u32> = expr(any((var(1),)));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_nested_expressions() {
        let mut x: Expr<u32> = expr(any((
            all((var(1), var(2))),
            var(3),
            all((var(1), var(2))),
            var(3),
        )));
        let expected: Expr<u32> = expr(any((all((var(1), var(2))), var(3))));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_with_constants() {
        let mut x: Expr<u32> = expr(any((const_(true), var(1), const_(true), const_(false))));
        let expected: Expr<u32> = expr(any((const_(true), var(1), const_(false))));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_preserves_order() {
        let mut x: Expr<u32> = expr(any((var(3), var(1), var(2), var(1), var(3))));
        let expected: Expr<u32> = expr(any((var(3), var(1), var(2))));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn dedup_non_list_expression() {
        let mut x: Expr<u32> = expr(not(var(1)));
        let expected: Expr<u32> = expr(not(var(1)));

        DedupList.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }
}
