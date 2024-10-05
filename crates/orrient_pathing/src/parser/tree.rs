use petgraph::{prelude::*, visit::Walker};

use super::pack::MarkerPath;

#[derive(Debug, Default)]
pub struct Trees<N> {
    pub roots: Vec<NodeIndex>,
    graph: DiGraph<N, ()>,
}

impl<N> Trees<N> {
    pub fn new() -> Self {
        Self {
            roots: Default::default(),
            graph: Default::default(),
        }
    }

    pub(super) fn graph(&self) -> &DiGraph<N, ()> {
        &self.graph
    }

    pub fn get(&self, idx: NodeIndex) -> Option<&N> {
        self.graph.node_weight(idx)
    }

    pub fn path_to(&self, idx: NodeIndex) -> Vec<NodeIndex> {
        let mut path = vec![idx];
        let mut current = self
            .graph
            .neighbors_directed(idx, Direction::Incoming)
            .next();
        while let Some(idx) = current {
            path.push(idx);
            current = self
                .graph
                .neighbors_directed(idx, Direction::Incoming)
                .next();
        }
        path
    }

    /// Walk down the tree following `path` to the destination.
    pub fn find<'a>(&'a self, path: impl TreePath<'a>) -> Option<(NodeIndex, &'a N)> {
        let mut path = path.into_iter();
        let mut current: NodeIndex = *path.next()?;
        for next in path {
            current = self
                .graph
                .neighbors_directed(current, Direction::Outgoing)
                .find(|neighbor| neighbor == next)?;
        }
        self.graph
            .node_weight(current)
            .map(|weight| (current, weight))
    }

    /// Walk down the tree following `path` to the destination.
    pub fn find_mut<'a>(&'a mut self, path: impl TreePath<'a>) -> Option<(NodeIndex, &'a mut N)> {
        let mut path = path.into_iter();
        let mut current: NodeIndex = *path.next()?;
        for next in path {
            current = self
                .graph
                .neighbors_directed(current, Direction::Outgoing)
                .find(|neighbor| neighbor == next)?;
        }
        self.graph
            .node_weight_mut(current)
            .map(|weight| (current, weight))
    }

    /// Merge `b` into `a` by moving all edges from `b` to `a`.
    pub fn merge(&mut self, a: NodeIndex, b: NodeIndex)
    where
        N: Clone,
    {
        for neighbor in self
            .graph
            .neighbors_directed(b, Direction::Outgoing)
            .collect::<Vec<_>>()
        {
            self.graph.add_edge(a, neighbor, ());
        }

        self.roots.retain(|a| a != &b);
        self.graph.remove_node(b);
    }

    /// Iterate through all direct neighbors of `start`.
    pub fn children(&self, start: NodeIndex) -> impl Iterator<Item = (NodeIndex, &N)> {
        self.graph
            .neighbors_directed(start, Direction::Outgoing)
            .filter_map(|idx| self.graph.node_weight(idx).map(|weight| (idx, weight)))
    }

    /// Recurse through all nodes starting at `start`.
    pub fn recurse(&self, start: NodeIndex) -> impl Iterator<Item = (NodeIndex, &N)> {
        petgraph::visit::Bfs::new(&self.graph, start)
            .iter(&self.graph)
            .filter_map(|idx| self.graph.node_weight(idx).map(|weight| (idx, weight)))
    }
}

pub trait TreePath<'a>: IntoIterator<Item = &'a NodeIndex> {}
impl<'a, T> TreePath<'a> for T where T: IntoIterator<Item = &'a NodeIndex> {}

pub struct TreeBuilder<N> {
    trees: Trees<N>,
    parents: Vec<NodeIndex>,
}

impl<N> std::ops::Deref for TreeBuilder<N> {
    type Target = Trees<N>;

    fn deref(&self) -> &Self::Target {
        &self.trees
    }
}

impl<N> std::ops::DerefMut for TreeBuilder<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.trees
    }
}

impl<N> TreeBuilder<N> {
    pub fn new() -> Self {
        Self {
            trees: Trees::<N>::new(),
            parents: Default::default(),
        }
    }

    /// Insert an item under the current parent and set this new item
    /// as the next parent.
    pub fn insert(&mut self, item: N) -> NodeIndex {
        let node_id = self.trees.graph.add_node(item);
        if let Some(parent_id) = self.parents.last() {
            self.trees.graph.add_edge(*parent_id, node_id, ());
        } else {
            self.trees.roots.push(node_id);
        }
        self.parents.push(node_id);
        node_id
    }

    /// Move up the tree removing the most recent parent from the
    /// list.
    pub fn new_root(&mut self) {
        self.parents.clear();
    }

    /// Move up the tree removing the most recent parent from the
    /// list.
    pub fn up(&mut self) {
        self.parents.pop();
    }

    /// Get the current path in the tree.
    pub fn path(&self) -> MarkerPath {
        MarkerPath(self.parents.clone())
    }

    pub fn build(self) -> Trees<N> {
        self.trees
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::dot::{Config, Dot};
    use std::io::Write as _;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Node(usize);

    fn write_dot(graph: &DiGraph<Node, ()>) {
        let dot = Dot::with_config(graph, &[Config::EdgeNoLabel]);
        let mut file = std::fs::File::create("/tmp/test.dot").unwrap();
        file.write(format!("{:?}", dot).as_bytes()).unwrap();
    }

    #[test]
    fn find() {
        let mut builder = TreeBuilder::new();
        let n_0 = &builder.insert(Node(0));
        let n_1 = &builder.insert(Node(1));
        let n_2 = &builder.insert(Node(2));
        builder.up();
        let n_3 = &builder.insert(Node(3));
        let n_4 = &builder.insert(Node(4));

        let n_10 = &builder.insert_root(Node(10));
        let n_11 = &builder.insert(Node(11));
        let _n_12 = &builder.insert(Node(12));
        builder.up();
        let n_13 = &builder.insert(Node(13));
        let n_14 = &builder.insert(Node(14));

        let tree = builder.build();
        write_dot(&tree.graph);

        assert!(tree.find([n_0, n_2]).is_none());
        assert!(tree.find([n_0, n_3]).is_none());
        assert!(tree.find([n_0, n_4]).is_none());

        assert_eq!(tree.find([n_0, n_1]).unwrap().1 .0, 1);
        assert_eq!(tree.find([n_0, n_1, n_2]).unwrap().1 .0, 2);
        assert_eq!(tree.find([n_0, n_1, n_3]).unwrap().1 .0, 3);
        assert_eq!(tree.find([n_0, n_1, n_3, n_4]).unwrap().1 .0, 4);

        assert!(tree.find([n_10, n_2]).is_none());
        assert_eq!(tree.find([n_10, n_11, n_13, n_14]).unwrap().1 .0, 14);
    }

    #[test]
    fn roots() {
        let mut builder = TreeBuilder::new();
        let n_0 = &builder.insert(Node(0));
        let _n_1 = &builder.insert(Node(1));
        assert_eq!(builder.trees.roots, vec![*n_0]);

        builder.up();
        builder.up();
        let n_2 = &builder.insert(Node(2));
        assert_eq!(builder.trees.roots, vec![*n_0, *n_2]);

        let n_3 = &builder.insert_root(Node(3));
        let tree = builder.build();
        assert_eq!(tree.roots, vec![*n_0, *n_2, *n_3]);
    }

    #[test]
    fn test_recurse() {
        let mut builder = TreeBuilder::new();
        let n_0 = builder.insert(Node(0));
        let n_1 = builder.insert(Node(1));
        let n_2 = builder.insert(Node(2));
        builder.up();
        let n_3 = builder.insert(Node(3));
        let n_4 = builder.insert(Node(4));
        let tree = builder.build();
        write_dot(&tree.graph);

        let mut iter = tree.recurse(n_1);
        assert_eq!(iter.next(), Some((n_1, &Node(1))));
        assert_eq!(iter.next(), Some((n_3, &Node(3))));
        assert_eq!(iter.next(), Some((n_2, &Node(2))));
        assert_eq!(iter.next(), Some((n_4, &Node(4))));
    }
}
