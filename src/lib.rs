pub mod container;
pub mod core;

use container::{
    traits::ContainerTrait,
    container::GenericContainer
};
use crate::core::seal::Contract;

pub fn region<Container, F, Out>(container: Container, f: F) -> Out
where
    F: for<'id> FnOnce(GenericContainer<Contract<'id>, Container>) -> Out,
    Container: ContainerTrait
{
    f(GenericContainer::new(container))
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

                let (lhs, rhs) = s.split_at(mid);

                assert_eq!(lhs[lhs.range()], [1, 2, 3]);
                assert_eq!(rhs[rhs.range()], [4, 5, 6]);

                let middle = lhs.range().nonempty().unwrap().upper_middle();
                let (llhs, lrhs) = lhs.split_at(middle);

                s[mid] = 9;
                assert_eq!(llhs[llhs.range()], [1]);
                assert_eq!(lrhs[lrhs.range()], [2, 3]);


            }

            assert!(triggered);
        })
    }
}