pub mod model;
pub mod trail;

use std::collections::{HashSet, VecDeque};
use std::convert::identity;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::ops::Deref;
use std::str::FromStr;
use std::{collections::HashMap, path::Path};

use log::{debug, warn};
use model::{Poi, Position};
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{Dfs, VisitMap};
use petgraph::Direction;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use typed_path::{Utf8PathBuf, Utf8UnixEncoding, Utf8WindowsPathBuf};

#[derive(Debug)]
pub enum Error {
    EmptyCategory,
    IoErr(std::io::Error),
    ZipErr(zip::result::ZipError),
    Eof,
    Xml(quick_xml::Error),
    MissingField(String),
    TrailParseError(String),
    UnknownField(String),
    AttrErr(quick_xml::events::attributes::AttrError),
    Utf8Error(std::string::FromUtf8Error),
}

pub fn read(directory: &Path) -> Result<MarkerTree, Error> {
    let mut tree = MarkerTreeBuilder::new(directory);

    let iter = std::fs::read_dir(directory).unwrap();
    for path in iter
        .filter_map(|file| file.ok().map(|file| file.path()))
        .filter(|file| file.is_file())
        .filter(|file| {
            file.extension()
                .map(|ext| ext == "taco" || ext == "zip")
                .unwrap_or_default()
        })
    {
        read_pack(&mut tree, &path).unwrap();
    }

    Ok(tree.build())
}

#[derive(Debug)]
enum Tag {
    OverlayData,
    Marker(Marker),
    POIs,
    POI(model::Poi),
    Trail(model::Trail),
    Route,
    UnknownField(String),
    CorruptField(String),
}

impl Tag {
    fn from_element(element: &BytesStart) -> Result<Tag, Error> {
        let tag = match element.name().0 {
            b"OverlayData" => Tag::OverlayData,
            b"MarkerCategory" => Tag::Marker(Marker::from_attrs(element.attributes())?),
            b"POIs" => Tag::POIs,
            b"POI" => Tag::POI(model::Poi::from_attrs(element.attributes())?),
            b"Trail" => Tag::Trail(model::Trail::from_attrs(element.attributes())?),
            field => Tag::UnknownField(String::from_utf8_lossy(field).to_string()),
        };

        Ok(tag)
    }
}

fn read_pack(tree: &mut MarkerTreeBuilder, path: &Path) -> Result<(), Error> {
    let pack = File::open(path).map_err(Error::IoErr)?;
    let mut zip = zip::ZipArchive::new(pack).map_err(Error::ZipErr)?;
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(Error::ZipErr)?;
        let filename = file.name().to_string();
        let Some(ext) = filename.rsplit(".").next() else {
            continue;
        };
        match ext {
            "xml" => {
                let _ = parse_xml(tree, &filename, BufReader::new(file));
            }
            _ => (),
        }
    }
    Ok(())
}

fn parse_xml<R: Read + BufRead>(
    tree: &mut MarkerTreeBuilder,
    filename: &str,
    reader: R,
) -> Result<(), Error> {
    let mut reader = Reader::from_reader(reader);
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(event) => match event {
                Event::Start(element) => {
                    tree.add_tag(match Tag::from_element(&element) {
                        Ok(tag) => tag,
                        Err(err) => {
                            warn!(
                                "Error parsing tag {:?} in file {:?}: {:?}",
                                &element, filename, err
                            );
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
                                &element, filename, err
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
                filename,
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

struct MarkerTreeBuilder<'a> {
    data_dir: &'a Path,

    tree: MarkerTree,

    /// The number of indices in the graph so to generate unique
    /// indices.
    count: usize,

    /// The path in the tree we currently are located.
    parent_id: VecDeque<NodeIndex>,
}

impl Deref for MarkerTreeBuilder<'_> {
    type Target = MarkerTree;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl<'a> MarkerTreeBuilder<'a> {
    fn new(data_dir: &'a Path) -> Self {
        Self {
            data_dir,
            tree: Default::default(),
            count: Default::default(),
            parent_id: Default::default(),
        }
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

    fn add_poi(&mut self, poi: Poi) {
        let id: MarkerID = poi.id.clone().into();
        if let Some(pois) = self.tree.pois.get_mut(&id) {
            pois.push(poi);
        } else {
            self.tree.pois.insert(id, vec![poi]);
        }
    }

    fn add_trail(&mut self, trail_tag: model::Trail) -> Option<Trail> {
        let id: MarkerID = trail_tag.id.into();
        let content = match trail::from_file(self.data_dir.join(&trail_tag.trail_file)) {
            Ok(content) => content,
            Err(err) => {
                warn!(
                    "Error while parsing trail at {}: {:?}",
                    trail_tag.trail_file, err
                );
                return None;
            }
        };

        let trail = Trail {
            map_id: content.map_id,
            path: content.path,
            texture_file: trail_tag.texture_file,
        };

        if let Some(trails) = self.tree.trails.get_mut(&id) {
            trails.push(trail.clone());
        } else {
            self.tree.trails.insert(id, vec![trail.clone()]);
        }
        Some(trail)
    }

    fn add_map_id(&mut self, id: impl Into<MarkerID>, map_id: u32) {
        if let Some(marker) = self.tree.get_mut(id) {
            marker.map_ids.push(map_id);
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
                self.add_map_id(poi.id.clone(), poi.map_id);
                self.add_poi(poi);
            }
            Tag::Trail(trail) => {
                let id: MarkerID = trail.id.clone().into();
                if let Some(trail) = self.add_trail(trail) {
                    self.add_map_id(id, trail.map_id);
                }
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
    pois: HashMap<MarkerID, Vec<Poi>>,

    /// Trails associated with markers
    trails: HashMap<MarkerID, Vec<Trail>>,
}

impl MarkerTree {
    fn new() -> Self {
        Self::default()
    }

    fn index_of(&self, id: impl Into<MarkerID>) -> Option<NodeIndex> {
        self.indexes.get(&id.into()).cloned()
    }

    pub fn contains_map_id(&self, id: impl Into<MarkerID>, map_id: u32) -> bool {
        for marker in self.iter_recursive(id) {
            if marker.map_ids.contains(&map_id) {
                return true;
            }
        }
        false
    }

    pub fn get_pois(&self, id: impl Into<MarkerID>) -> Option<&Vec<Poi>> {
        self.pois.get(&id.into())
    }

    pub fn get_trails(&self, id: impl Into<MarkerID>) -> Option<&Vec<Trail>> {
        self.trails.get(&id.into())
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

#[derive(Clone, Debug)]
pub struct Trail {
    pub map_id: u32,
    pub path: Vec<Position>,
    pub texture_file: String,
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
    pub map_ids: Vec<u32>,
    pub icon_file: Option<Utf8PathBuf<Utf8UnixEncoding>>,
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
        if self.icon_file.is_none() {
            self.icon_file = parent.icon_file.clone();
        }
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
                "iconfile" => {
                    if let Ok(path) = Utf8WindowsPathBuf::from_str(&value) {
                        this.icon_file = Some(path.with_unix_encoding().to_path_buf());
                    } else {
                        warn!("Icon path is corrupt: {:?}", attr)
                    }
                }
                _ => {}
            }
        }
        Ok(this)
    }
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
        let mut markers = MarkerTreeBuilder::new(Path::new(""));
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
