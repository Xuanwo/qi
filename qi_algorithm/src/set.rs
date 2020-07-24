use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct OrderedSet<T> {
    index: HashMap<T, usize>,
    pub items: Vec<T>,
    length: usize,
}

impl<T: Hash + Eq + Clone> OrderedSet<T> {
    pub fn new() -> OrderedSet<T> {
        OrderedSet {
            index: HashMap::new(),
            items: Vec::new(),
            length: 0,
        }
    }

    pub fn add(&mut self, item: &T) -> bool {
        if self.index.contains_key(&item) {
            return false;
        }

        self.index.insert(item.clone(), self.length.into());
        self.items.push(item.clone());
        self.length += 1;

        true
    }

    pub fn index(&self, item: &T) -> Option<usize> {
        self.index.get(item).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_set() {
        let mut x = OrderedSet::new();

        let added = x.add(&1);
        assert_eq!(added, true);

        let not_added = x.add(&1);
        assert_eq!(not_added, false);

        let idx = x.index(&1);
        assert_eq!(idx.unwrap(), 0);

        let not_exist = x.index(&2);
        assert_eq!(not_exist.is_none(), true)
    }
}
