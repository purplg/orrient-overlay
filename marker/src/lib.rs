mod marker;
pub mod trail;

use std::{collections::HashMap, path::Path};

use petgraph::{
    graphmap::DiGraphMap,
    visit::{Dfs, VisitMap},
    Direction,
};

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

    let mut builder = MarkerTreeBuilder::new_from_file(root);

    // Loop through markers to populate any associated Trails or POIs.
    for (id, marker) in &mut builder.markers {
        marker.trail_file = data
            .pois
            .iter()
            .find_map(|poi| {
                poi.trail
                    .iter()
                    .find(|trail| trail.id.split(".").last() == Some(id))
            })
            .map(|trail| trail.trail_data.clone());

        for pois in &data.pois {
            for poi in pois.poi.iter() {
                if poi.id.split(".").last() == Some(id) {
                    marker.pois.push(Position {
                        x: poi.x,
                        y: poi.y,
                        z: poi.z,
                    });
                }
            }
        }
    }

    Ok(builder.build())
}

struct MarkerTreeBuilder {
    root: &'static str,
    nodes: Vec<&'static str>,
    edges: Vec<(&'static str, &'static str)>,
    markers: HashMap<&'static str, Marker>,
}

impl MarkerTreeBuilder {
    fn new_empty(root: &'static str) -> Self {
        Self {
            root,
            nodes: Default::default(),
            edges: Default::default(),
            markers: Default::default(),
        }
    }

    fn new_from_file(category: &marker::MarkerCategory) -> Self {
        let mut builder = Self::new_empty(category.id().leak());
        builder.insert_category_recursive(category, 0, None);
        builder
    }

    fn insert_marker(&mut self, id: &'static str, marker: Marker, parent: Option<&Marker>) {
        self.markers.insert(id, marker);

        self.nodes.push(id);

        if let Some(parent) = parent {
            self.edges.push((parent.id.clone().leak(), id));
        }
    }

    fn insert_category_recursive(
        &mut self,
        category: &marker::MarkerCategory,
        depth: usize,
        parent: Option<&Marker>,
    ) {
        let id: &str = category.id().leak();
        let mut marker = Marker::new(
            id,
            category.display_name(),
            if category.is_separator {
                MarkerKind::Separator
            } else {
                if category.categories.len() == 0 {
                    MarkerKind::Leaf
                } else {
                    MarkerKind::Category
                }
            },
            depth,
        );

        if let Some(parent) = parent {
            marker.copy_from_parent(parent);
        }

        marker.poi_tip = category.tip_name.clone();
        marker.poi_description = category.tip_description.clone();
        marker.behavior = Behavior::from_category(&category);

        self.insert_marker(id, marker.clone(), parent);

        for subcat in &category.categories {
            self.insert_category_recursive(&subcat, depth + 1, Some(&marker));
        }
    }

    fn build(mut self) -> MarkerTree {
        let mut graph: DiGraphMap<&'static str, ()> = Default::default();
        for node in self.nodes.drain(..) {
            graph.add_node(node);
        }
        for (a, b) in self.edges.drain(..) {
            graph.add_edge(a, b, ());
        }

        MarkerTree {
            root: self.root,
            graph,
            markers: self.markers,
        }
    }
}

pub struct MarkerTreeIter<'a, VM: VisitMap<&'a str>> {
    tree: &'a MarkerTree,
    iter: Dfs<&'a str, VM>,
}

impl<'a, VM: VisitMap<&'a str>> Iterator for MarkerTreeIter<'a, VM> {
    type Item = MarkerTreeItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next(&self.tree.graph).and_then(|id| {
            self.tree.markers.get(id).map(|marker| MarkerTreeItem {
                id,
                marker,
                depth: self.tree.get(id).unwrap().depth,
            })
        })
    }
}

pub struct MarkerTreeItem<'a> {
    pub id: &'a str,
    pub marker: &'a Marker,
    pub depth: usize,
}

#[derive(Clone, Debug)]
pub struct MarkerTree {
    pub root: &'static str,
    graph: DiGraphMap<&'static str, ()>,
    markers: HashMap<&'static str, Marker>,
}

impl MarkerTree {
    pub fn get(&self, id: &str) -> Option<&Marker> {
        self.markers.get(id)
    }

    pub fn root<'a>(&'a self) -> Option<MarkerTreeItem<'a>> {
        self.get(self.root).map(|marker| MarkerTreeItem {
            id: self.root,
            marker,
            depth: marker.depth,
        })
    }

    pub fn iter<'a>(&'a self, start: &'a str) -> impl Iterator<Item = MarkerTreeItem<'a>> {
        self.graph
            .neighbors_directed(start, Direction::Outgoing)
            .filter_map(|id| {
                self.markers.get(id).map(|marker| MarkerTreeItem {
                    id,
                    marker,
                    depth: marker.depth,
                })
            })
    }

    pub fn iter_recursive<'a>(
        &'a self,
        start: Option<&'a str>,
    ) -> impl Iterator<Item = MarkerTreeItem<'a>> {
        MarkerTreeIter {
            tree: self,
            iter: Dfs::new(&self.graph, start.unwrap_or(self.root)),
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
    pub pois: Vec<Position>,
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
            pois: Default::default(),
            trail_file: Default::default(),
        }
    }

    fn copy_from_parent(&mut self, parent: &Marker) {
        self.behavior = parent.behavior;
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
        let mut markers = MarkerTreeBuilder::new_empty("A");
        markers.insert_marker(None, "A", Marker::category("A", 0));
        markers.insert_marker(Some("A"), "B", Marker::category("B", 1));
        markers.insert_marker(Some("B"), "C", Marker::category("C", 2));
        markers.insert_marker(Some("B"), "D", Marker::category("D", 2));
        markers.insert_marker(Some("A"), "E", Marker::category("E", 1));
        markers.insert_marker(Some("E"), "F", Marker::category("F", 2));
        markers.build()
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
        let mut iter = markers.iter_recursive();
        assert_eq!(iter.next().unwrap().id, "A");
        assert_eq!(iter.next().unwrap().id, "B");
        assert_eq!(iter.next().unwrap().id, "C");
        assert_eq!(iter.next().unwrap().id, "D");
        assert_eq!(iter.next().unwrap().id, "E");
        assert_eq!(iter.next().unwrap().id, "F");
    }

    // #[test]
    fn test_get_path() {
        let markers = fake_markers();
        assert_eq!(markers.get_path(vec!["1"]).unwrap().label, "1 name");
        assert_eq!(
            markers.get_path(vec!["1", "1.1"]).unwrap().label,
            "1.1 name"
        );
        assert_eq!(
            markers.get_path(vec!["1", "1.2"]).unwrap().label,
            "1.2 name"
        );
    }

    // #[test]
    fn test_real_get_path() {
        let markers: MarkerTree = read(Path::new(
            "/home/purplg/.config/orrient/markers/tw_lws03e05_draconismons.xml",
        ))
        .unwrap();

        markers
            .get_path(vec![
                "tw_guides",
                "tw_lws3",
                "tw_lws3_draconismons",
                "tw_lws3_draconismons_primordialorchids",
                "tw_lws3_draconismons_primordialorchids_toggletrail",
                "tw_lws3_draconismons_primordialorchids_toggletrail_p1",
            ])
            .unwrap();
    }
}
