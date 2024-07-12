use bevy::prelude::*;
use quick_xml::events::attributes::Attributes;
use std::{collections::VecDeque, convert::identity, ops::Deref, str::FromStr as _};
use typed_path::{Utf8PathBuf, Utf8UnixEncoding, Utf8WindowsPathBuf};

use bevy::utils::{HashMap, HashSet};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, VisitMap},
    Direction,
};

use super::{
    model::{self, Poi},
    Error, PackId,
};

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq)]
pub struct MarkerId(pub String);

impl std::fmt::Display for MarkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq)]
pub struct FullMarkerId {
    pub pack_id: super::PackId,
    pub marker_id: MarkerId,
}

impl FullMarkerId {
    pub fn with_marker_id(&self, marker_id: MarkerId) -> FullMarkerId {
        FullMarkerId {
            pack_id: self.pack_id.clone(),
            marker_id,
        }
    }
}

impl std::fmt::Display for FullMarkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pack_id.fmt(f);
        ":".fmt(f);
        self.marker_id.fmt(f)
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
    pub map_ids: Vec<u32>,
    pub icon_file: Option<Utf8PathBuf<Utf8UnixEncoding>>,
}

impl Marker {
    pub(super) fn new(id: impl Into<String>, label: impl Into<String>, kind: MarkerKind) -> Self {
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

    pub fn from_attrs(attrs: Attributes) -> Result<Self, Error> {
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

#[derive(Clone, Debug)]
pub struct Trail {
    pub map_id: u32,
    pub path: Vec<Vec3>,
    pub texture_file: String,
}

#[derive(Clone, Default, Debug)]
pub struct MarkerPack {
    /// Nodes without any parents. Useful for iterating through all
    /// content in the graph.
    roots: HashSet<NodeIndex>,

    /// Lookup an index by it's string ID
    indexes: HashMap<MarkerId, NodeIndex>,

    /// Lookup a marker by its index
    markers: HashMap<NodeIndex, Marker>,

    /// Map the relationship between markers
    graph: DiGraph<NodeIndex, ()>,

    /// POIs associated with markers
    pois: HashMap<MarkerId, Vec<Poi>>,

    /// Trails associated with markers
    trails: HashMap<MarkerId, Vec<Trail>>,

    icons: HashMap<String, Handle<Image>>,
}

impl MarkerPack {
    fn new() -> Self {
        Self::default()
    }

    fn index_of(&self, id: &MarkerId) -> Option<NodeIndex> {
        self.indexes.get(id).cloned()
    }

    pub fn contains_map_id(&self, id: &MarkerId, map_id: u32) -> bool {
        for marker in self.iter_recursive(id) {
            if marker.map_ids.contains(&map_id) {
                return true;
            }
        }
        false
    }

    pub fn get_pois(&self, id: &MarkerId) -> Option<&Vec<Poi>> {
        self.pois.get(id)
    }

    pub fn get_icon(&self, path: &str) -> Option<Handle<Image>> {
        self.icons.get(path).cloned()
    }

    pub fn get_trails(&self, id: &MarkerId) -> Option<&Vec<Trail>> {
        self.trails.get(id)
    }

    pub fn get(&self, id: &MarkerId) -> Option<&Marker> {
        let node_id = self.indexes.get(id)?;
        self.markers.get(node_id)
    }

    pub fn get_mut(&mut self, id: &MarkerId) -> Option<&mut Marker> {
        let node_id = self.indexes.get(id).unwrap();
        self.markers.get_mut(node_id)
    }

    pub fn roots(&self) -> impl Iterator<Item = &Marker> {
        self.roots
            .iter()
            .filter_map(|index| self.markers.get(index))
    }

    pub fn iter<'a>(&'a self, start: &MarkerId) -> impl Iterator<Item = &'a Marker> {
        let start_id = self.index_of(start).unwrap();
        self.graph
            .neighbors_directed(start_id, Direction::Outgoing)
            .filter_map(|id| self.markers.get(&id))
    }

    pub fn iter_recursive<'a>(&'a self, start: &MarkerId) -> impl Iterator<Item = &'a Marker> {
        let start_id = self.index_of(start).unwrap();
        MarkerPackIter {
            tree: self,
            iter: Dfs::new(&self.graph, start_id),
        }
    }
}

pub struct MarkerPackIter<'a, VM: VisitMap<NodeIndex>> {
    tree: &'a MarkerPack,
    iter: Dfs<NodeIndex, VM>,
}

impl<'a, VM: VisitMap<NodeIndex>> Iterator for MarkerPackIter<'a, VM> {
    type Item = &'a Marker;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next(&self.tree.graph)
            .and_then(|id| self.tree.markers.get(&id))
    }
}

pub struct MarkerPackBuilder {
    id: PackId,

    tree: MarkerPack,

    /// The number of indices in the graph so to generate unique
    /// indices.
    count: usize,

    /// The path in the tree we currently are located.
    parent_id: VecDeque<NodeIndex>,
}

impl Deref for MarkerPackBuilder {
    type Target = MarkerPack;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl MarkerPackBuilder {
    pub fn new(pack_id: PackId) -> Self {
        Self {
            id: pack_id,
            tree: Default::default(),
            count: Default::default(),
            parent_id: Default::default(),
        }
    }

    pub fn add_marker(&mut self, mut marker: Marker) -> &mut Self {
        let node_id = self.get_or_create_index(&MarkerId(marker.id.clone()));
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
        self.tree.indexes.insert(MarkerId(marker.id), node_id);
        self
    }

    pub fn add_poi(&mut self, poi: Poi) {
        let id: MarkerId = MarkerId(poi.id.clone());

        if let Some(pois) = self.tree.pois.get_mut(&id) {
            pois.push(poi);
        } else {
            self.tree.pois.insert(id, vec![poi]);
        }
    }

    pub fn add_trail(&mut self, id: impl Into<MarkerId>, trail: Trail) {
        let id: MarkerId = id.into();
        // let content = match trail::from_file(self.data_dir.join(&trail_tag.trail_file)) {
        //     Ok(content) => content,
        //     Err(err) => {
        //         warn!(
        //             "Error while parsing trail at {}: {:?}",
        //             trail_tag.trail_file, err
        //         );
        //         return None;
        //     }
        // };

        // let trail = Trail {
        //     map_id: content.map_id,
        //     path: content.path,
        //     texture_file: trail_tag.texture_file,
        // };

        if let Some(trails) = self.tree.trails.get_mut(&id) {
            trails.push(trail.clone());
        } else {
            self.tree.trails.insert(id, vec![trail.clone()]);
        }
    }

    pub fn add_image(&mut self, filename: String, image: Image, image_assets: &mut Assets<Image>) {
        let handle = image_assets.add(image);
        self.tree.icons.insert(filename, handle);
    }

    pub fn add_map_id(&mut self, id: impl Into<String>, map_id: u32) {
        if let Some(marker) = self.tree.get_mut(&MarkerId(id.into())) {
            marker.map_ids.push(map_id);
        }
    }

    fn get_or_create_index(&mut self, marker_id: &MarkerId) -> NodeIndex {
        self.tree.index_of(&marker_id).unwrap_or_else(|| {
            NodeIndex::new({
                let i = self.count;
                self.count += 1;
                i
            })
        })
    }

    pub fn up(&mut self) {
        self.parent_id.pop_front();
    }

    pub fn new_root(&mut self) {
        self.parent_id.clear();
    }

    pub fn build(self) -> MarkerPack {
        self.tree
    }
}
