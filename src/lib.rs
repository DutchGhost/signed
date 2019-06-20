#![feature(nll)]
pub mod container;
pub mod core;

use crate::core::seal::Signed;
use container::{traits::ContainerTrait, Container};

pub fn region<C, F, Out>(container: C, f: F) -> Out
where
    F: for<'id> FnOnce(Container<Signed<'id>, C>) -> Out,
    C: ContainerTrait,
{
    f(Container::new(container))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region() {
        let mut v = vec![1, 2, 3, 4, 5, 6];

        region(v.as_mut_slice(), |mut s| {
            let mut triggered = false;
            if let Some(r) = s.range().nonempty() {
                triggered = true;
                let mid = r.upper_middle();

                let (lhs, rhs) = s.split_at_mut(mid);

                region(lhs, |lhsc| {
                    assert_eq!(lhsc[lhsc.range()], [1, 2, 3]);
                });

                region(rhs, |rhsc| {
                    assert_eq!(rhsc[rhsc.range()], [4, 5, 6]);
                })
            }
        })
    }

    #[test]
    fn test_scan_from() {
        let mut v = vec![1, 2, 3, 4, 5, 6, 7];

        region(v.as_mut_slice(), |mut s| {
            if let Some(r) = s.range().nonempty() {
                let mid = r.upper_middle();
                assert_eq!(mid.integer(), 3);
                let scanned_range = s.scan_from(mid, |x| x < &3);

                assert_eq!(s[scanned_range], [4]);
            }
        })
    }

    #[test]
    fn test_split_first() {
        let mut v = vec![1, 2, 3, 4, 5, 6, 7];

        region(v.as_mut_slice(), |mut s| {
            let (first, rest) = s.split_first().unwrap();

            assert_eq!(first, &1);
            assert_eq!(rest[..], [2, 3, 4, 5, 6, 7]);

            let (first, rest) = s.split_first_mut().unwrap();

            assert_eq!(first, &mut 1);
            assert_eq!(rest[..], [2, 3, 4, 5, 6, 7]);
        })
    }

    #[test]
    fn test_split_first_size_is_1() {
        let mut v = vec![1];

        region(v.as_mut_slice(), |mut s| {
            let (first, rest) = s.split_first().unwrap();

            assert_eq!(first, &1);
            assert_eq!(rest[..], []);
            assert!(rest.is_empty());

            let (first, rest) = s.split_first_mut().unwrap();

            assert_eq!(first, &mut 1);
            assert_eq!(rest[..], []);
            assert!(rest.is_empty());
        })
    }

    #[test]
    fn test_split_first_size_is_0() {
        let mut v: Vec<usize> = vec![];

        region(v.as_mut_slice(), |mut s| {
            assert!(s.split_first().is_none());
            assert!(s.split_first_mut().is_none());
        })
    }

    #[test]
    fn test_split_index_into_other() {
        let mut v = vec![1, 2, 3, 4, 5, 6];

        region(v.as_mut_slice(), |mut s| {
            if let Some(r) = s.range().nonempty() {
                let mid = r.upper_middle();

                let (lhs, rhs) = s.split_at(mid);
            }
        })
    }
}
