use crate::ast::{All, Any, Expr, Not};
use crate::visit_mut::VisitMut;
use crate::visit_mut::walk_mut_expr;

pub struct FlattenByDeMorgan;

impl<T> VisitMut<T> for FlattenByDeMorgan {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_mut_expr(self, expr);

        if let Expr::Not(Not(not)) = expr {
            match &mut **not {
                Expr::Any(Any(any)) => {
                    let list = any.drain(..).map(|x| Expr::Not(Not(Box::new(x)))).collect();
                    *expr = Expr::All(All(list));
                }
                Expr::All(All(all)) => {
                    let list = all.drain(..).map(|x| Expr::Not(Not(Box::new(x)))).collect();
                    *expr = Expr::Any(Any(list));
                }
                _ => {}
            }
        }

        walk_mut_expr(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, const_, expr, not, var};

    #[test]
    fn de_morgan_not_any_to_all() {
        // not(any(a, b)) -> all(not(a), not(b))
        let mut x: Expr<u32> = expr(not(any((var(1), var(2)))));
        let expected: Expr<u32> = expr(all((not(var(1)), not(var(2)))));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_not_all_to_any() {
        // not(all(a, b)) -> any(not(a), not(b))
        let mut x: Expr<u32> = expr(not(all((var(1), var(2)))));
        let expected: Expr<u32> = expr(any((not(var(1)), not(var(2)))));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_not_empty_any() {
        // not(any()) -> all()
        let mut x: Expr<u32> = expr(not(any(())));
        let expected: Expr<u32> = expr(all(()));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_not_empty_all() {
        // not(all()) -> any()
        let mut x: Expr<u32> = expr(not(all(())));
        let expected: Expr<u32> = expr(any(()));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_not_single_any() {
        // not(any(a)) -> all(not(a))
        let mut x: Expr<u32> = expr(not(any((var(1),))));
        let expected: Expr<u32> = expr(all((not(var(1)),)));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_not_single_all() {
        // not(all(a)) -> any(not(a))
        let mut x: Expr<u32> = expr(not(all((var(1),))));
        let expected: Expr<u32> = expr(any((not(var(1)),)));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_not_multiple_any() {
        // not(any(a, b, c)) -> all(not(a), not(b), not(c))
        let mut x: Expr<u32> = expr(not(any((var(1), var(2), var(3)))));
        let expected: Expr<u32> = expr(all((not(var(1)), not(var(2)), not(var(3)))));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_not_multiple_all() {
        // not(all(a, b, c)) -> any(not(a), not(b), not(c))
        let mut x: Expr<u32> = expr(not(all((var(1), var(2), var(3)))));
        let expected: Expr<u32> = expr(any((not(var(1)), not(var(2)), not(var(3)))));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_change_not_variable() {
        // not(a) -> not(a) (no change)
        let mut x: Expr<u32> = expr(not(var(1)));
        let expected: Expr<u32> = expr(not(var(1)));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_change_not_const() {
        // not(true) -> not(true) (no change)
        let mut x: Expr<u32> = expr(not(const_(true)));
        let expected: Expr<u32> = expr(not(const_(true)));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_nested() {
        // not(any(all(a, b), c)) -> all(not(all(a, b)), not(c)) -> all(any(not(a), not(b)), not(c))
        let mut x: Expr<u32> = expr(not(any((all((var(1), var(2))), var(3)))));
        let expected: Expr<u32> = expr(all((any((not(var(1)), not(var(2)))), not(var(3)))));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn de_morgan_complex_nested() {
        // not(all(any(a, b), any(c, d))) -> any(not(any(a, b)), not(any(c, d))) -> any(all(not(a), not(b)), all(not(c), not(d)))
        let mut x: Expr<u32> = expr(not(all((any((var(1), var(2))), any((var(3), var(4)))))));
        let expected: Expr<u32> = expr(any((
            all((not(var(1)), not(var(2)))),
            all((not(var(3)), not(var(4)))),
        )));

        FlattenByDeMorgan.visit_mut_expr(&mut x);

        assert_eq!(x.to_string(), expected.to_string());
    }
}
