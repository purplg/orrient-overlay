mod model;
pub mod trail;

use std::collections::{HashSet, VecDeque};
use std::convert::identity;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;
use std::{collections::HashMap, path::Path};

use log::{debug, info, warn};
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, VisitMap};
use petgraph::Direction;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::QName;
use quick_xml::Reader;

#[derive(Debug)]
pub enum Error {
    EmptyCategory,
    IoErr(std::io::Error),
    Xml(quick_xml::Error),
    FieldErr { field: String, message: String },
    UnknownField(String),
    AttrErr(quick_xml::events::attributes::AttrError),
    Utf8Error(std::string::FromUtf8Error),
}

pub fn read(directory: &Path) -> Result<MarkerTree, Error> {
    let mut tree = MarkerTreeBuilder::default();

    let iter = std::fs::read_dir(directory).unwrap();
    for path in iter
        .filter_map(|file| file.ok().map(|file| file.path()))
        .filter(|file| file.is_file())
        .filter(|file| file.extension().map(|ext| ext == "xml").unwrap_or_default())
    {
        read_file(&mut tree, &path).unwrap();
    }

    println!("tree.roots: {:?}", tree.roots());

    Ok(tree.build())
}

#[derive(Debug)]
enum Tag {
    OverlayData,
    Marker(Marker),
    POIs,
    POI(model::Poi),
    Route,
    UnknownField(String),
    CorruptField(String),
}

impl Tag {
    fn from_element(element: &BytesStart) -> Result<Tag, Error> {
        let tag = match element.name() {
            QName(b"OverlayData") => Tag::OverlayData,
            QName(b"MarkerCategory") => Tag::Marker(Marker::from_attrs(element.attributes())?),
            QName(b"POIs") => Tag::POIs,
            QName(b"POI") => Tag::POI(model::Poi::from_attrs(element.attributes())?),
            QName(field) => Tag::UnknownField(String::from_utf8_lossy(field).to_string()),
        };

        Ok(tag)
    }
}

fn read_file(tree: &mut MarkerTreeBuilder, path: &Path) -> Result<(), Error> {
    let mut reader = Reader::from_file(path).map_err(Error::Xml)?;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(event) => match event {
                Event::Start(element) => {
                    tree.add_tag(match Tag::from_element(&element) {
                        Ok(tag) => tag,
                        Err(err) => {
                            warn!("Error parsing tag in file {:?}: {:?}", path, err);
                            continue;
                        }
                    });
                }
                Event::Empty(element) => {
                    tree.add_tag(match Tag::from_element(&element) {
                        Ok(tag) => tag,
                        Err(err) => {
                            warn!(
                                "Error parsing tag {:?} in file {:?}: {:?}",
                                &element, path, err
                            );
                            continue;
                        }
                    });
                    tree.up();
                }
                Event::End(_) => {
                    tree.up();
                }
                Event::Eof => break,
                unknown_event => debug!("unknown_event: {:?}", unknown_event),
            },
            Err(err) => panic!(
                "Error reading {:?} at position {}: {:?}",
                path,
                reader.buffer_position(),
                err
            ),
        }
    }

    tree.new_root();
    Ok(())
}

pub struct MarkerTreeIter<'a, VM: VisitMap<NodeIndex>> {
    tree: &'a MarkerTree,
    iter: Dfs<NodeIndex, VM>,
}

impl<'a, VM: VisitMap<NodeIndex>> Iterator for MarkerTreeIter<'a, VM> {
    type Item = &'a Marker;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next(&self.tree.graph)
            .and_then(|id| self.tree.markers.get(&id))
    }
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub struct MarkerID(String);

impl Deref for MarkerID {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a str> for MarkerID {
    fn from(value: &'a str) -> Self {
        MarkerID(value.to_owned())
    }
}

impl<'a> From<&'a String> for MarkerID {
    fn from(value: &'a String) -> Self {
        MarkerID(value.to_owned())
    }
}

impl From<String> for MarkerID {
    fn from(value: String) -> Self {
        MarkerID(value)
    }
}

#[derive(Default, Debug)]
struct MarkerTreeBuilder {
    tree: MarkerTree,

    /// The number of indices in the graph so to generate unique
    /// indices.
    count: usize,

    /// The path in the tree we currently are located.
    parent_id: VecDeque<NodeIndex>,
}

impl Deref for MarkerTreeBuilder {
    type Target = MarkerTree;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl MarkerTreeBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn add_marker(&mut self, mut marker: Marker) -> &mut Self {
        let node_id = self.get_or_create_index(&marker.id);
        self.tree.graph.add_node(node_id);

        if let Some(parent_id) = self.parent_id.front() {
            self.tree.graph.add_edge(*parent_id, node_id, ());
            let parent_marker = self.tree.markers.get(parent_id).unwrap();
            marker.copy_from_parent(parent_marker);
        } else {
            self.tree.roots.insert(node_id);
        }

        self.parent_id.push_front(node_id);
        self.tree.markers.insert(node_id, marker.clone());
        self.tree.indexes.insert(marker.id.into(), node_id);
        self
    }

    fn add_poi(&mut self, id: impl Into<MarkerID>, x: f32, y: f32, z: f32) {
        let position = Position { x, y, z };
        let id = id.into();
        if let Some(pois) = self.tree.pois.get_mut(&id) {
            pois.push(position);
        } else {
            self.tree.pois.insert(id, vec![position]);
        }
    }

    fn get_or_create_index(&mut self, marker_id: impl Into<MarkerID>) -> NodeIndex {
        self.tree.index_of(marker_id).unwrap_or_else(|| {
            NodeIndex::new({
                let i = self.count;
                self.count += 1;
                i
            })
        })
    }

    fn up(&mut self) {
        self.parent_id.pop_front();
    }

    fn add_tag(&mut self, tag: Tag) {
        match tag {
            Tag::OverlayData => {
                self.new_root();
            }
            Tag::Marker(marker) => {
                self.add_marker(marker);
            }
            Tag::POIs => {}
            Tag::POI(poi) => {
                self.add_poi(poi.id, poi.x, poi.y, poi.z);
            }
            Tag::Route => {}
            Tag::UnknownField(_) => {}
            Tag::CorruptField(_) => todo!(),
        }
    }

    fn new_root(&mut self) {
        self.parent_id.clear();
    }

    fn build(self) -> MarkerTree {
        self.tree
    }
}

#[derive(Clone, Default, Debug)]
pub struct MarkerTree {
    /// Nodes without any parents. Useful for iterating through all
    /// content in the graph.
    roots: HashSet<NodeIndex>,

    /// Lookup an index by it's string ID
    indexes: HashMap<MarkerID, NodeIndex>,

    /// Lookup a marker by its index
    markers: HashMap<NodeIndex, Marker>,

    /// Map the relationship between markers
    graph: DiGraph<NodeIndex, ()>,

    /// POIs associated with markers
    pois: HashMap<MarkerID, Vec<Position>>,
}

impl MarkerTree {
    fn new() -> Self {
        Self::default()
    }

    // fn insert_category_recursive(
    //     &mut self,
    //     category: &marker::MarkerCategory,
    //     depth: usize,
    //     parent_id: Option<NodeIndex>,
    // ) {
    //     let kind = if category.is_separator {
    //         MarkerKind::Separator
    //     } else if category.categories.is_empty() {
    //         MarkerKind::Leaf
    //     } else {
    //         MarkerKind::Category
    //     };

    //     let parent = parent_id.and_then(|parent_id: NodeIndex| self.markers.get(&parent_id));
    //     let mut marker = Marker::new(category.id, category.display_name, kind, depth);
    //     if let Some(parent) = parent {
    //         marker.copy_from_parent(parent);
    //     }

    //     marker.poi_tip = category.tip_name.clone();
    //     marker.poi_description = category.tip_description.clone();
    //     marker.behavior = Behavior::from_category(category);

    //     let node_id = self.insert_marker(marker.clone(), parent_id);

    //     if parent_id.is_none() {
    //         self.roots.insert(node_id);
    //     }

    //     for subcat in &category.categories {
    //         self.insert_category_recursive(subcat, depth + 1, Some(node_id));
    //     }
    // }

    fn index_of(&self, id: impl Into<MarkerID>) -> Option<NodeIndex> {
        self.indexes.get(&id.into()).cloned()
    }

    pub fn get_pois(&self, id: impl Into<MarkerID>) -> Option<&Vec<Position>> {
        self.pois.get(&id.into())
    }

    pub fn get(&self, id: impl Into<MarkerID>) -> Option<&Marker> {
        let node_id = self.indexes.get(&id.into()).unwrap();
        self.markers.get(node_id)
    }

    pub fn get_mut(&mut self, id: impl Into<MarkerID>) -> Option<&mut Marker> {
        let node_id = self.indexes.get(&id.into()).unwrap();
        self.markers.get_mut(node_id)
    }

    pub fn roots(&self) -> Vec<&Marker> {
        self.roots
            .iter()
            .filter_map(|index| self.markers.get(index))
            .collect()
    }

    pub fn iter<'a>(&'a self, start: impl Into<MarkerID>) -> impl Iterator<Item = &'a Marker> {
        let start_id = self.index_of(start).unwrap();
        self.graph
            .neighbors_directed(start_id, Direction::Outgoing)
            .filter_map(|id| self.markers.get(&id))
    }

    pub fn iter_recursive<'a>(
        &'a self,
        start: impl Into<MarkerID>,
    ) -> impl Iterator<Item = &'a Marker> {
        let start_id = self.index_of(start).unwrap();
        MarkerTreeIter {
            tree: self,
            iter: Dfs::new(&self.graph, start_id),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum MarkerKind {
    #[default]
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
    fn from_category(category: model::MarkerCategory) -> Option<Behavior> {
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

#[derive(Clone, Debug, Default)]
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
    fn new(id: impl Into<String>, label: impl Into<String>, kind: MarkerKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            ..Default::default()
        }
    }

    fn copy_from_parent(&mut self, parent: &Marker) {
        self.id = format!("{}.{}", parent.id, self.id);
        self.depth = parent.depth + 1;
        self.behavior = self.behavior.or(parent.behavior);
    }

    fn from_attrs(attrs: Attributes) -> Result<Self, Error> {
        let mut this = Self::default();

        for attr in attrs.map(Result::ok).filter_map(identity) {
            let Ok(key) = String::from_utf8(attr.key.0.to_vec()) else {
                warn!("Key is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            let Ok(value) = String::from_utf8(attr.value.to_vec()) else {
                warn!("Value is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            match key.to_lowercase().as_str() {
                "name" => this.id = value,
                "displayname" => this.label = value,
                "isseparator" => {
                    if "true" == value.to_lowercase() {
                        this.kind = MarkerKind::Separator
                    };
                }
                _ => {}
            }
        }
        Ok(this)
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

    impl Marker {
        fn category(id: impl Into<String>, label: impl Into<String>) -> Self {
            Self::new(id, label, MarkerKind::Category)
        }
    }

    //     A            G
    //    / \          / \
    //   B   E        H   I
    //  / \   \         / | \
    // C   D   F       J  K  L
    fn fake_markers() -> MarkerTree {
        let mut markers = MarkerTreeBuilder::new();
        markers.add_marker(Marker::category("A", "A Name"));
        markers.add_marker(Marker::category("B", "B Name"));
        markers.add_marker(Marker::category("C", "C Name"));
        markers.up();
        markers.add_marker(Marker::category("D", "D Name"));
        markers.up();
        markers.up();
        markers.add_marker(Marker::category("E", "E Name"));
        markers.add_marker(Marker::category("F", "F Name"));

        markers.new_root();
        markers.add_marker(Marker::category("G", "G Name"));
        markers.add_marker(Marker::category("H", "H Name"));
        markers.up();
        markers.add_marker(Marker::category("I", "I Name"));
        markers.add_marker(Marker::category("J", "J Name"));
        markers.up();
        markers.add_marker(Marker::category("K", "K Name"));
        markers.up();
        markers.add_marker(Marker::category("L", "L Name"));
        markers.build()
    }

    #[test]
    fn test_real_data() {
        env_logger::init();

        read(&dirs::config_dir().unwrap().join("orrient").join("markers")).unwrap();
    }

    #[test]
    fn test_iter() {
        let markers = fake_markers();
        println!(
            "markers: {:?}",
            markers
                .roots()
                .iter()
                .map(|a| a.id.clone())
                .collect::<Vec<_>>()
        );
        let mut iter = markers.iter_recursive("A");

        //     A
        //    / \
        //   B   E
        //  / \   \
        // C   D   F
        assert_eq!(iter.next().unwrap().id, "A");
        assert_eq!(iter.next().unwrap().id, "A.B");
        assert_eq!(iter.next().unwrap().id, "A.B.C");
        assert_eq!(iter.next().unwrap().id, "A.B.D");
        assert_eq!(iter.next().unwrap().id, "A.E");
        assert_eq!(iter.next().unwrap().id, "A.E.F");
        assert!(iter.next().is_none());

        //   G
        //  / \
        // H   I
        //   / | \
        //  J  K  L
        let mut iter = markers.iter_recursive("G");
        assert_eq!(iter.next().unwrap().id, "G");
        assert_eq!(iter.next().unwrap().id, "G.H");
        assert_eq!(iter.next().unwrap().id, "G.I");
        assert_eq!(iter.next().unwrap().id, "G.I.J");
        assert_eq!(iter.next().unwrap().id, "G.I.K");
        assert_eq!(iter.next().unwrap().id, "G.I.L");
        assert!(iter.next().is_none());

        //     A
        //    / \
        //   B   E
        //  / \   \
        // C   D   F
        let mut iter = markers.iter_recursive("A.B");
        assert_eq!(iter.next().unwrap().id, "A.B");
        assert_eq!(iter.next().unwrap().id, "A.B.C");
        assert_eq!(iter.next().unwrap().id, "A.B.D");
        assert!(iter.next().is_none());

        //     A
        //    / \
        //   B   E
        //  / \   \
        // C   D   F
        let mut iter = markers.iter_recursive("A.B.C");
        assert_eq!(iter.next().unwrap().id, "A.B.C");
        assert!(iter.next().is_none());
    }
}
