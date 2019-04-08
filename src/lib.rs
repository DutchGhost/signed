pub mod container;
pub mod core;

use crate::core::seal::Signed;
use container::{container::Container, traits::ContainerTrait};

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

                assert_eq!(lhs[lhs.range()], [1, 2, 3]);
                assert_eq!(rhs[rhs.range()], [4, 5, 6]);

                let middle = lhs.range().nonempty().unwrap().upper_middle();
                let (llhs, lrhs) = lhs.split_at(middle);

                assert_eq!(llhs[llhs.range()], [1]);
                assert_eq!(lrhs[lrhs.range()], [2, 3]);
            }

            assert!(triggered);
        })
    }

    #[test]
    fn test_scan_from() {
        let mut v = vec![1, 2, 3, 4, 5, 6, 7];

        region(v.as_mut_slice(), |s| {
            if let Some(r) = s.range().nonempty() {
                let mid = r.upper_middle();
                assert_eq!(mid.integer(), 3);
                let scanned_range = s.scan_from(mid, |x| x < &3);

                assert_eq!(s[scanned_range], [4]);
            }
        })
    }
}
