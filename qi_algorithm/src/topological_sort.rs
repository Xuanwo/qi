use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::fmt::Debug;

use super::set::OrderedSet;

struct Node<T> (HashSet<T>);

impl<T> Deref for Node<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T: Hash + Eq + Clone + Debug> Node<T> {
    fn new() -> Node<T> {
        Node(HashSet::new())
    }

    fn add_edge(&mut self, name: &T) {
        self.insert(name.clone());
    }
}

struct Topology<T> (HashMap<T, Node<T>>);

impl<T> Deref for Topology<T> {
    type Target = HashMap<T, Node<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Topology<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T: Hash + Eq + Clone + Debug> Topology<T> {
    fn new() -> Topology<T> {
        Topology(HashMap::new())
    }

    fn add_node(&mut self, name: &T) {
        if !self.contains_key(name) {
            self.insert(name.clone(), Node::new());
        }
    }

    fn add_edge(&mut self, from: &T, to: &T) {
        self.add_node(&from);
        self.add_node(&to);

        self.get_mut(&from).unwrap().add_edge(to);
    }

    fn visit(&self, name: &T, results: &mut OrderedSet<T>, mut visited: OrderedSet<T>) -> Result<(), String> {
        let added = visited.add(name);
        if !added {
            return Err(format!("cycle error, {:?}", visited.items));
        }

        let n = self.get(&name).unwrap();
        for edge in n.iter() {
            self.visit(edge, results, visited.clone())?;
        }

        results.add(&name);

        Ok(())
    }

    fn sort(&mut self, name: &T) -> Result<Vec<T>, String> {
        let mut results: OrderedSet<T> = OrderedSet::new();

        self.visit(name, &mut results, OrderedSet::new())?;

        Ok(results.items)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn init_topology() -> Topology<String> {
        let mut graph = Topology::new();
        graph.add_node(&String::from("a"));
        graph.add_node(&String::from("b"));
        graph.add_node(&String::from("c"));
        graph.add_node(&String::from("d"));

        graph
    }

    #[test]
    fn empty_parent() {
        let mut graph = Topology::new();

        let sa = String::from("a");
        let sb = String::from("b");

        graph.add_edge(&sa, &sb);

        let results = match graph.sort(&sa) {
            Err(e) => {
                panic!(e)
            }
            Ok(x) => x
        };

        assert_eq!(results.get(0).unwrap(), &sb);
    }

    #[test]
    fn top_sort_1() {
        let mut graph = init_topology();

        let (a, b, c) = (
            &"a".to_string(),
            &"b".to_string(),
            &"c".to_string(),
        );

        // a -> b -> c
        graph.add_edge(a, b);
        graph.add_edge(b, c);

        let results = match graph.sort(a) {
            Err(e) => {
                panic!(e)
            }
            Ok(x) => x
        };

        assert_eq!(results, vec![c.to_owned(), b.to_owned(), a.to_owned()]);
    }

    #[test]
    fn top_sort_2() {
        let mut graph = init_topology();

        let (a, b, c) = (
            &"a".to_string(),
            &"b".to_string(),
            &"c".to_string(),
        );


        // a -> c
        // a -> b
        // b -> c
        graph.add_edge(a, c);
        graph.add_edge(a, b);
        graph.add_edge(b, c);


        let results = match graph.sort(a) {
            Err(e) => {
                panic!(e)
            }
            Ok(x) => x
        };

        assert_eq!(results, vec![c.to_owned(), b.to_owned(), a.to_owned()]);
    }

    #[test]
    fn top_sort_3() {
        let mut graph = init_topology();

        let (a, b, c, d) = (
            &"a".to_string(),
            &"b".to_string(),
            &"c".to_string(),
            &"d".to_string(),
        );


        // a -> b
        // a -> d
        // d -> c
        // c -> b
        graph.add_edge(a, b);
        graph.add_edge(a, d);
        graph.add_edge(d, c);
        graph.add_edge(c, b);


        let results = match graph.sort(a) {
            Err(e) => {
                panic!(e)
            }
            Ok(x) => x
        };

        assert_eq!(results, vec![b.to_owned(), c.to_owned(), d.to_owned(), a.to_owned()]);
    }
}