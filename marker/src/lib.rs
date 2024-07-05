mod marker;
pub mod trail;

use std::{collections::HashMap, path::Path};

use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, VisitMap};
use petgraph::Direction;

#[derive(Debug)]
pub enum Error {
    EmptyCategory,
    FsErr(std::io::Error),
    DeErr(quick_xml::de::DeError),
}

pub fn read(path: &Path) -> Result<MarkerTree, Error> {
    let content = std::fs::read_to_string(path).map_err(Error::FsErr)?;
    let data: marker::OverlayData = quick_xml::de::from_str(&content).map_err(Error::DeErr)?;

    let Some(root) = data.categories.first() else {
        return Err(Error::EmptyCategory);
    };

    let mut tree = MarkerTree::new_from_file(root);

    // Loop through markers to populate any associated Trails or POIs.
    for (index, marker) in &mut tree.markers {
        marker.trail_file = data
            .pois
            .iter()
            .find_map(|poi| {
                poi.trail.iter().find(|trail| {
                    let Some(trail_id) = trail.id.split(".").last() else {
                        return false;
                    };
                    tree.indexes
                        .get(&trail_id.to_string())
                        .map(|node_index| node_index == index)
                        .unwrap_or_default()
                })
            })
            .map(|marker| marker.trail_data.clone());
    }

    for pois in &data.pois {
        for poi in pois.poi.iter() {
            let Some(poi_id) = poi.id.split(".").last() else {
                continue;
            };

            tree.add_poi(
                poi_id.to_string(),
                Position {
                    x: poi.x,
                    y: poi.y,
                    z: poi.z,
                },
            );
        }
    }

    Ok(tree)
}

pub struct MarkerTreeIter<'a, VM: VisitMap<NodeIndex>> {
    tree: &'a MarkerTree,
    iter: Dfs<NodeIndex, VM>,
}

impl<'a, VM: VisitMap<NodeIndex>> Iterator for MarkerTreeIter<'a, VM> {
    type Item = &'a Marker;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next(&self.tree.graph).and_then(|id| {
            self.tree.markers.get(&id)
        })
    }
}

#[derive(Clone, Default, Debug)]
pub struct MarkerTree {
    count: usize,
    roots: Vec<NodeIndex>,
    indexes: HashMap<String, NodeIndex>,
    markers: HashMap<NodeIndex, Marker>,
    graph: DiGraph<NodeIndex, ()>,
    pois: HashMap<String, Vec<Position>>,
}

impl MarkerTree {
    fn new() -> Self {
        Self::default()
    }

    fn new_from_file(category: &marker::MarkerCategory) -> Self {
        let mut builder = Self::new();
        builder.insert_category_recursive(category, 0, None);
        builder
    }

    fn insert_marker(&mut self, marker: Marker, parent: Option<NodeIndex>) -> NodeIndex {
        let node_id = NodeIndex::new(self.count);
        self.count += 1;

        self.markers.insert(node_id, marker.clone());
        self.indexes.insert(marker.id, node_id);

        self.graph.add_node(node_id);

        if let Some(parent) = parent {
            self.graph.add_edge(parent, node_id, ());
        }

        node_id
    }

    fn insert_category_recursive(
        &mut self,
        category: &marker::MarkerCategory,
        depth: usize,
        parent_id: Option<NodeIndex>,
    ) {
        let kind = if category.is_separator {
            MarkerKind::Separator
        } else {
            if category.categories.len() == 0 {
                MarkerKind::Leaf
            } else {
                MarkerKind::Category
            }
        };

        let parent = parent_id.and_then(|parent_id: NodeIndex| self.markers.get(&parent_id));
        let mut marker = if let Some(parent) = parent {
            Marker::new_from_parent(category.id(), category.display_name(), kind, parent)
        } else {
            Marker::new(category.id(), category.display_name(), kind, depth)
        };

        marker.poi_tip = category.tip_name.clone();
        marker.poi_description = category.tip_description.clone();
        marker.behavior = Behavior::from_category(&category);

        let node_id = self.insert_marker(marker.clone(), parent_id);

        if parent_id.is_none() {
            self.roots.push(node_id);
        }

        for subcat in &category.categories {
            self.insert_category_recursive(&subcat, depth + 1, Some(node_id));
        }
    }

    fn add_poi(&mut self, id: String, position: Position) {
        if let Some(mut pois) = self.pois.get_mut(&id) {
            pois.push(position);
        } else {
            self.pois.insert(id, vec![position]);
        }
    }

    pub fn get_pois(&self, id: &String) -> Option<&Vec<Position>> {
        self.pois.get(id)
    }

    pub fn get(&self, id: &str) -> Option<&Marker> {
        let node_id = self.indexes.get(id).unwrap();
        self.markers.get(node_id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Marker> {
        let node_id = self.indexes.get(id).unwrap();
        self.markers.get_mut(node_id)
    }

    pub fn roots(&self) -> Vec<&Marker> {
        self.roots
            .iter()
            .filter_map(|index| self.markers.get(index))
            .collect()
    }

    pub fn iter<'a>(&'a self, start: &'a str) -> impl Iterator<Item = &'a Marker> {
        let start_id = self.indexes.get(start).unwrap();
        self.graph
            .neighbors_directed(*start_id, Direction::Outgoing)
            .filter_map(|id| self.markers.get(&id))
    }

    pub fn iter_recursive<'a>(&'a self, start: &'a str) -> impl Iterator<Item = &'a Marker> {
        let start_id = self.indexes.get(start).unwrap();
        MarkerTreeIter {
            tree: self,
            iter: Dfs::new(&self.graph, *start_id),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MarkerKind {
    Category,
    Leaf,
    Separator,
}

#[derive(Clone, Copy, Debug)]
pub enum Behavior {
    AlwaysVisible,             // 0
    ReappearOnMapChange,       // 1
    ReappearDaily,             // 2
    DisappearOnUse,            // 3
    ReappearAfterTime(f32),    // 4
    ReappearMapReset,          // 5
    ReappearInstanceChange,    // 6
    ReappearDailyPerCharacter, // 7
}

impl Behavior {
    fn from_category(category: &marker::MarkerCategory) -> Option<Behavior> {
        if let Some(behavior) = category.behavior {
            match behavior {
                0 => Some(Self::AlwaysVisible),
                1 => Some(Self::ReappearOnMapChange),
                2 => Some(Self::ReappearDaily),
                3 => Some(Self::DisappearOnUse),
                4 => Some(Self::ReappearAfterTime(
                    category
                        .reset_length
                        .expect("resetLength must be defined to use Behavior 4"),
                )),
                5 => Some(Self::ReappearMapReset),
                6 => Some(Self::ReappearInstanceChange),
                7 => Some(Self::ReappearDailyPerCharacter),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Marker {
    pub id: String,
    pub label: String,
    pub kind: MarkerKind,
    pub depth: usize,
    pub behavior: Option<Behavior>,
    pub poi_tip: Option<String>,
    pub poi_description: Option<String>,
    pub trail_file: Option<String>,
}

impl Marker {
    fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: MarkerKind,
        depth: usize,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            depth,
            behavior: Default::default(),
            poi_tip: Default::default(),
            poi_description: Default::default(),
            trail_file: Default::default(),
        }
    }

    fn category(id: impl Into<String>, label: impl Into<String>, depth: usize) -> Self {
        Self::new(id, label, MarkerKind::Category, depth)
    }

    fn new_from_parent(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: MarkerKind,
        parent: &Marker,
    ) -> Self {
        let mut marker = Self::new(id, label, kind, parent.depth);
        marker.behavior = parent.behavior;
        marker
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    //     A
    //    / \
    //   B   E
    //  / \   \
    // C   D   F
    fn fake_markers() -> MarkerTree {
        let mut markers = MarkerTree::new();
        let a_id = markers.insert_marker(Marker::category("A", "A Name", 0), None);
        let b_id = markers.insert_marker(Marker::category("B", "B Name", 1), Some(a_id));
        let c_id = markers.insert_marker(Marker::category("C", "C Name", 2), Some(b_id));
        let d_id = markers.insert_marker(Marker::category("D", "D Name", 2), Some(b_id));
        let e_id = markers.insert_marker(Marker::category("E", "E Name", 1), Some(a_id));
        let f_id = markers.insert_marker(Marker::category("F", "F Name", 2), Some(e_id));

        let g_id = markers.insert_marker(Marker::category("G", "G Name", 0), None);
        let h_id = markers.insert_marker(Marker::category("H", "H Name", 1), Some(g_id));
        let i_id = markers.insert_marker(Marker::category("I", "I Name", 2), Some(h_id));
        let j_id = markers.insert_marker(Marker::category("J", "J Name", 2), Some(h_id));
        let k_id = markers.insert_marker(Marker::category("K", "K Name", 1), Some(g_id));
        let l_id = markers.insert_marker(Marker::category("L", "L Name", 2), Some(k_id));
        markers
    }

    // #[test]
    fn test_real_data() {
        let iter = std::fs::read_dir("/home/purplg/.config/orrient/markers").unwrap();
        for path in iter
            .filter_map(|file| file.ok().map(|file| file.path()))
            .filter(|file| file.is_file())
            .filter(|file| file.extension().map(|ext| ext == "xml").unwrap_or_default())
        {
            read(&path).unwrap();
        }
    }

    #[test]
    fn test_iter() {
        let markers = fake_markers();
        let mut iter = markers.iter_recursive("A");
        assert_eq!(iter.next().unwrap().id, "A");
        assert_eq!(iter.next().unwrap().id, "B");
        assert_eq!(iter.next().unwrap().id, "C");
        assert_eq!(iter.next().unwrap().id, "D");
        assert_eq!(iter.next().unwrap().id, "E");
        assert_eq!(iter.next().unwrap().id, "F");
        assert!(iter.next().is_none());

        let mut iter = markers.iter_recursive("G");
        assert_eq!(iter.next().unwrap().id, "G");
        assert_eq!(iter.next().unwrap().id, "H");
        assert_eq!(iter.next().unwrap().id, "I");
        assert_eq!(iter.next().unwrap().id, "J");
        assert_eq!(iter.next().unwrap().id, "K");
        assert_eq!(iter.next().unwrap().id, "L");
        assert!(iter.next().is_none());

        let mut iter = markers.iter_recursive("B");
        assert_eq!(iter.next().unwrap().id, "B");
        assert_eq!(iter.next().unwrap().id, "C");
        assert_eq!(iter.next().unwrap().id, "D");
        assert!(iter.next().is_none());

        let mut iter = markers.iter_recursive("C");
        assert_eq!(iter.next().unwrap().id, "C");
        assert!(iter.next().is_none());
    }

    // #[test]
    // fn test_real_get_path() {
    //     let markers: MarkerTree = read(Path::new(
    //         "/home/purplg/.config/orrient/markers/tw_lws03e05_draconismons.xml",
    //     ))
    //     .unwrap();

    //     markers
    //         .get_path(vec![
    //             "tw_guides",
    //             "tw_lws3",
    //             "tw_lws3_draconismons",
    //             "tw_lws3_draconismons_primordialorchids",
    //             "tw_lws3_draconismons_primordialorchids_toggletrail",
    //             "tw_lws3_draconismons_primordialorchids_toggletrail_p1",
    //         ])
    //         .unwrap();
    // }
}
