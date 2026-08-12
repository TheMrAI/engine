use std::collections::VecDeque;

#[derive(Debug, Default)]
pub struct Cyclid<StorageType> {
    items: Vec<Option<StorageType>>,
    released: VecDeque<usize>,
}

impl<StorageType> Cyclid<StorageType> {
    pub fn insert(&mut self, item: StorageType) -> usize {
        let id = match self.released.is_empty() {
            true => self.items.len(),
            false => self.released.pop_front().unwrap(),
        };
        self.items.push(Some(item));
        id
    }

    pub fn get(&self, index: usize) -> &StorageType {
        self.items[index].as_ref().unwrap()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut StorageType {
        self.items.get_mut(index).unwrap().as_mut().unwrap()
    }

    pub fn release(&mut self, index: usize) -> StorageType {
        let element = self.items.get_mut(index).unwrap().take();
        self.released.push_back(index);
        element.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Cyclid;

    #[test]
    fn insert_get() {
        let mut indexes = Cyclid::<u32>::default();
        let id_zero = indexes.insert(0);
        let id_one = indexes.insert(1);
        let id_two = indexes.insert(2);

        assert_eq!(0, *indexes.get(id_zero));
        assert_eq!(1, *indexes.get(id_one));
        assert_eq!(2, *indexes.get(id_two));
        let element_one = indexes.release(id_one);
        assert_eq!(1, element_one);
        let id_three = indexes.insert(3);
        assert_eq!(id_three, 1);
    }
}
