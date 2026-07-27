use crate::ComponentId;

/// The sequence is be kept in non-decreasing order and
/// the elements are pairwise-distinct.
#[derive(Debug, Default, Clone, Hash, Eq, PartialEq)]
pub struct DynamicSignature {
    components: Vec<ComponentId>,
}

// TODO impl Hash such that it simply returns the stored hash
// update the stored hash every time a component is added or removed.

impl DynamicSignature {
    pub fn new() -> Self {
        Self {
            components: Default::default(),
        }
    }

    pub fn from_slice(component_slice: &[ComponentId]) -> Self {
        let components = {
            let mut components = Into::<Vec<ComponentId>>::into(component_slice);
            components.sort();
            components.dedup();
            components
        };
        Self { components }
    }

    pub fn get(&self, index: usize) -> ComponentId {
        self.components[index]
    }

    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components().is_empty()
    }

    pub fn insert(&mut self, id: ComponentId) {
        match self.components.binary_search(&id) {
            Result::Ok(_) => (),
            Result::Err(index) => self.components.insert(index, id),
        }
    }

    pub fn remove(&mut self, id: ComponentId) -> bool {
        match self.components.binary_search(&id) {
            Result::Ok(index) => {
                self.components.remove(index);
                true
            }
            Result::Err(_) => false,
        }
    }
}

impl From<&[ComponentId]> for DynamicSignature {
    fn from(component_slice: &[ComponentId]) -> Self {
        DynamicSignature::from_slice(component_slice)
    }
}

#[cfg(test)]
mod tests {
    use crate::signature::DynamicSignature;

    fn basic_sequence() -> DynamicSignature {
        let mut signature = DynamicSignature::new();
        signature.insert(2);
        signature.insert(3);
        signature.insert(5);
        signature.insert(0);
        signature.insert(4);
        signature.insert(6);
        signature.insert(1);
        signature
    }

    #[test]
    fn is_ordered() {
        let signature = basic_sequence();
        assert_eq!(signature.len(), 7);
        assert_eq!(signature.components(), [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn no_duplicates() {
        let mut signature = basic_sequence();

        signature.insert(1);
        signature.insert(1);
        signature.insert(2);
        signature.insert(2);
        signature.insert(0);
        signature.insert(0);
        assert_eq!(signature.len(), 7);
        assert_eq!(signature.components(), [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn remove_elements() {
        let mut signature = basic_sequence();

        assert_eq!(signature.remove(1), true);
        assert_eq!(signature.remove(1), false);
        assert_eq!(signature.remove(0), true);
        assert_eq!(signature.remove(0), false);
        assert_eq!(signature.remove(6), true);
        assert_eq!(signature.remove(6), false);
        assert_eq!(signature.remove(3), true);
        assert_eq!(signature.remove(3), false);

        assert_eq!(signature.len(), 3);
        assert_eq!(signature.components(), [2, 4, 5]);
    }

    #[test]
    fn form_slice() {
        let signature = DynamicSignature::from_slice(&[1u32, 1, 1, 0, 2, 0, 2, 3, 1]);
        assert_eq!(signature.components(), &[0, 1, 2, 3]);
    }
}
