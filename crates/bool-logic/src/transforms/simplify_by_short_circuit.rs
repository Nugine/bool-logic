use std::ops::Not as _;

use crate::ast::{All, Any, Expr, Not, Var};
use crate::visit_mut::VisitMut;

fn find_vars<T: Eq + Clone>(list: &mut [Expr<T>], marker: bool) -> Vec<Var<T>> {
    let mut ans: Vec<Var<T>> = Vec::new();
    for x in list {
        if let Expr::Var(var) = x {
            if ans.contains(var) {
                *x = Expr::Const(marker);
            } else {
                ans.push(var.clone());
            }
        }
    }
    ans
}

fn replace_vars<T: Eq>(x: &mut Expr<T>, vars: &[Var<T>], marker: bool) {
    match x {
        Expr::Any(Any(any)) => any.iter_mut().for_each(|x| replace_vars(x, vars, marker)),
        Expr::All(All(all)) => all.iter_mut().for_each(|x| replace_vars(x, vars, marker)),
        Expr::Not(Not(not)) => replace_vars(not, vars, marker),
        Expr::Var(var) => {
            if vars.contains(var) {
                *x = Expr::Const(marker);
            }
        }
        Expr::Const(_) => {}
    }
}

pub struct SimplifyByShortCircuit;

impl<T: Eq + Clone> VisitMut<T> for SimplifyByShortCircuit {
    fn visit_mut_any(&mut self, Any(any): &mut Any<T>) {
        let marker = false;
        let vars = find_vars(any, marker);
        for x in any.iter_mut().filter(|x| x.is_var().not()) {
            replace_vars(x, &vars, marker);
        }
    }

    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        let marker = true;
        let vars = find_vars(all, marker);
        for x in all.iter_mut().filter(|x| x.is_var().not()) {
            replace_vars(x, &vars, marker);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, const_, expr, var};

    #[test]
    fn test_simplify_any_with_duplicates() {
        let mut expr_val = expr(any((
            var(1),
            var(2),
            var(1), // duplicate
            all((var(3), var(1))),
        )));

        SimplifyByShortCircuit.visit_mut_expr(&mut expr_val);

        let any_val = expr_val.as_any().unwrap();
        assert_eq!(any_val.0[2], expr(const_(false))); // duplicate replaced
        // Check that var(1) in nested All is also replaced
        let all_val = any_val.0[3].as_all().unwrap();
        assert_eq!(all_val.0[1], expr(const_(false)));
    }

    #[test]
    fn test_simplify_all_with_duplicates() {
        let mut expr_val = expr(all((
            var(1),
            var(2),
            var(1), // duplicate
            any((var(3), var(1))),
        )));

        SimplifyByShortCircuit.visit_mut_expr(&mut expr_val);

        let all_val = expr_val.as_all().unwrap();
        assert_eq!(all_val.0[2], expr(const_(true))); // duplicate replaced
        // Check that var(1) in nested Any is also replaced
        let any_val = all_val.0[3].as_any().unwrap();
        assert_eq!(any_val.0[1], expr(const_(true)));
    }

    #[test]
    fn test_no_duplicates() {
        let mut expr_val = expr(any((var(1), var(2), var(3))));
        let original = expr_val.clone();

        SimplifyByShortCircuit.visit_mut_expr(&mut expr_val);

        assert_eq!(expr_val.to_string(), original.to_string());
    }
}
