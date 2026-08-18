use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct OuterContainer {
    inner_map: HashMap<u32, Rc<RefCell<InnerContainer>>>,
}

impl OuterContainer {
    pub fn get(&self, key: u32) -> Rc<RefCell<InnerContainer>> {
        self.inner_map.get(&key).unwrap().clone()
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
    use super::{InnerContainer, OuterContainer};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn immutable_access_multiple_inner_entries() {
        let outer = OuterContainer {
            inner_map: HashMap::from([
                (0, Rc::new(RefCell::new(InnerContainer { data: vec![0] }))),
                (
                    1,
                    Rc::new(RefCell::new(InnerContainer { data: vec![0, 1] })),
                ),
                (
                    2,
                    Rc::new(RefCell::new(InnerContainer {
                        data: vec![0, 1, 2],
                    })),
                ),
            ]),
        };

        let zero = *outer.get(0).borrow().get(0);
        let one = *outer.get(1).borrow().get(1);
        let two = *outer.get(2).borrow().get(2);

        assert_eq!(zero, 0);
        assert_eq!(one, 1);
        assert_eq!(two, 2);
    }

    #[test]
    fn mutable_access_multiple_inner_entries() {
        let outer = OuterContainer {
            inner_map: HashMap::from([
                (0, Rc::new(RefCell::new(InnerContainer { data: vec![0] }))),
                (
                    1,
                    Rc::new(RefCell::new(InnerContainer { data: vec![0, 1] })),
                ),
                (
                    2,
                    Rc::new(RefCell::new(InnerContainer {
                        data: vec![0, 1, 2],
                    })),
                ),
            ]),
        };

        {
            let at_zero = outer.get(0);
            let first_vec = &mut at_zero.borrow_mut().data;
            first_vec[0] += 1;

            let at_one = outer.get(1);
            let second_vec = &mut at_one.borrow_mut().data;
            second_vec[1] += 2;

            let at_two = outer.get(2);
            let third_vec = &mut at_two.borrow_mut().data;
            third_vec[2] += 3;

            assert_eq!(first_vec.as_slice(), [1]);
            assert_eq!(second_vec.as_slice(), [0, 3]);
            assert_eq!(third_vec.as_slice(), [0, 1, 5]);
        }

        let another = outer.get(1);
        let second_again = &another.borrow().data;
        assert_eq!(second_again.as_slice(), [0, 3]);
    }
}
