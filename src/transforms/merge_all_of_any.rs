use crate::ast::{All, Any, Expr};
use crate::utils::remove_if;
use crate::visit_mut::{VisitMut, walk_mut_expr_list};

fn is_subset_of<T: Eq>(lhs: &[Expr<T>], rhs: &[Expr<T>]) -> bool {
    lhs.iter().all(|x| rhs.contains(x))
}

fn get2_mut<T>(slice: &mut [T], i: usize, j: usize) -> Option<(&mut T, &mut T)> {
    let [lhs, rhs] = slice.get_disjoint_mut([i, j]).ok()?;
    Some((lhs, rhs))
}

pub struct MergeAllOfAny;

impl<T: Eq> VisitMut<T> for MergeAllOfAny {
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        walk_mut_expr_list(self, all);

        let mut any_list: Vec<_> = all
            .iter_mut()
            .filter_map(|x| Expr::as_mut_any(x).map(|x| &mut x.0))
            .collect();

        for i in 0..any_list.len() {
            for j in 0..any_list.len() {
                if i != j {
                    let (lhs, rhs) = get2_mut(&mut any_list, i, j).unwrap();
                    if is_subset_of(lhs, rhs) {
                        rhs.clear();
                        rhs.push(Expr::Const(true));
                    }
                }
            }
        }

        remove_if(all, |x| match x {
            Expr::Any(Any(any)) => matches!(any.as_slice(), [Expr::Const(true)]),
            _ => false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{all, any, expr, var};

    #[test]
    fn merge_subset_any() {
        // all(any(a, b), any(a)) -> all(any(a))
        // any(a) is subset of any(a,b), so any(a,b) is redundant
        let mut x: Expr<u32> = expr(all((any((var(1), var(2))), any((var(1),)))));
        let expected: Expr<u32> = expr(all((any((var(1),)),)));

        MergeAllOfAny.visit_mut_all(Expr::as_mut_all(&mut x).unwrap());

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn merge_equal_any() {
        // all(any(a, b), any(a, b)) -> all(any(a, b))
        let mut x: Expr<u32> = expr(all((any((var(1), var(2))), any((var(1), var(2))))));
        let expected: Expr<u32> = expr(all((any((var(1), var(2))),)));

        MergeAllOfAny.visit_mut_all(Expr::as_mut_all(&mut x).unwrap());

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn merge_multiple_subsets() {
        // all(any(a, b, c), any(a, b), any(a)) -> all(any(a))
        // any(a) ⊆ any(a,b) ⊆ any(a,b,c), so keep the smallest
        let mut x: Expr<u32> = expr(all((
            any((var(1), var(2), var(3))),
            any((var(1), var(2))),
            any((var(1),)),
        )));
        let expected: Expr<u32> = expr(all((any((var(1),)),)));

        MergeAllOfAny.visit_mut_all(Expr::as_mut_all(&mut x).unwrap());

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn no_merge_non_subset() {
        // all(any(a, b), any(c, d)) -> all(any(a, b), any(c, d))
        let mut x: Expr<u32> = expr(all((any((var(1), var(2))), any((var(3), var(4))))));
        let expected: Expr<u32> = expr(all((any((var(1), var(2))), any((var(3), var(4))))));

        MergeAllOfAny.visit_mut_all(Expr::as_mut_all(&mut x).unwrap());

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn merge_with_mixed_expressions() {
        // all(var(1), any(a, b), any(a)) -> all(var(1), any(a))
        let mut x: Expr<u32> = expr(all((var(1), any((var(2), var(3))), any((var(2),)))));
        let expected: Expr<u32> = expr(all((var(1), any((var(2),)))));

        MergeAllOfAny.visit_mut_all(Expr::as_mut_all(&mut x).unwrap());

        assert_eq!(x.to_string(), expected.to_string());
    }

    #[test]
    fn merge_empty_subset() {
        // all(any(a, b), any()) -> all(any())
        // Empty any is subset of any non-empty any
        let mut x: Expr<u32> = expr(all((any((var(1), var(2))), any(()))));
        let expected: Expr<u32> = expr(all((any(()),)));

        MergeAllOfAny.visit_mut_all(Expr::as_mut_all(&mut x).unwrap());

        assert_eq!(x.to_string(), expected.to_string());
    }
}
