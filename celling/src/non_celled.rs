use std::collections::HashMap;

struct OuterContainer {
    inner_map: HashMap<u32, InnerContainer>,
}

impl OuterContainer {
    pub fn get(&self, key: u32, index: usize) -> &u32 {
        self.inner_map.get(&key).unwrap().get(index)
    }

    pub fn get_mut(&mut self, key: u32, index: usize) -> &mut u32 {
        self.inner_map.get_mut(&key).unwrap().get_mut(index)
    }
}

struct InnerContainer {
    data: Vec<u32>,
}

impl InnerContainer {
    pub fn get(&self, index: usize) -> &u32 {
        self.data.get(index).unwrap()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut u32 {
        self.data.get_mut(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{InnerContainer, OuterContainer};

    #[test]
    fn immutable_access_multiple_inner_entries() {
        let outer = OuterContainer {
            inner_map: HashMap::from([
                (0, InnerContainer { data: vec![0] }),
                (1, InnerContainer { data: vec![0, 1] }),
                (
                    2,
                    InnerContainer {
                        data: vec![0, 1, 2],
                    },
                ),
            ]),
        };

        let zero = outer.get(0, 0);
        let one = outer.get(1, 1);
        let two = outer.get(2, 2);

        assert_eq!(*zero, 0);
        assert_eq!(*one, 1);
        assert_eq!(*two, 2);
    }

    #[test]
    fn mutable_access_multiple_inner_entries() {
        let mut outer = OuterContainer {
            inner_map: HashMap::from([
                (0, InnerContainer { data: vec![0] }),
                (1, InnerContainer { data: vec![0, 1] }),
                (
                    2,
                    InnerContainer {
                        data: vec![0, 1, 2],
                    },
                ),
            ]),
        };

        let zero = outer.get_mut(0, 0);
        // fails, multiple mutable borrows to Outer
        // even though we don't care about Outer being
        // borrowed as the things we are borrowing are internal
        // and completely disjoint
        // let one = outer.get_mut(1, 1);
        // let two = outer.get_mut(2, 2);

        assert_eq!(*zero, 0);
        // assert_eq!(*one, 1);
        // assert_eq!(*two, 2);
    }
}
