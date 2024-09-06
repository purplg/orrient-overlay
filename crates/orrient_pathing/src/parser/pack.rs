use anyhow::Result;
use bevy::utils::HashMap;
use bevy::{prelude::*, utils::HashSet};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    Direction,
};
use quick_xml::events::attributes::Attributes;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::{collections::VecDeque, convert::identity, ops::Deref, str::FromStr as _};
use typed_path::{Utf8PathBuf, Utf8UnixEncoding, Utf8WindowsPathBuf};

use super::{
    model::{self, Poi, TrailXml},
    trail::TrailData,
    PackId,
};

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkerId(pub Cow<'static, str>);

impl MarkerId {
    /// Returns true when `other` is a child of this `MarkerId`.
    pub fn contains(&self, other: &MarkerId) -> bool {
        self != other && other.0.starts_with(&*self.0)
    }

    /// Returns true when `other` is a parent of this `MarkerId`.
    pub fn within(&self, other: &MarkerId) -> bool {
        self != other && self.0.starts_with(&*other.0)
    }
}

impl std::fmt::Display for MarkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Component, Hash, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullMarkerId {
    pub pack_id: PackId,
    pub marker_id: MarkerId,
}

impl FullMarkerId {
    pub fn with_marker_id(&self, marker_id: MarkerId) -> FullMarkerId {
        FullMarkerId {
            pack_id: self.pack_id.clone(),
            marker_id,
        }
    }

    /// Returns true when `other` is a child of this `FullMarkerId`.
    pub fn contains(&self, other: &FullMarkerId) -> bool {
        self.pack_id == other.pack_id && self.marker_id.contains(&other.marker_id)
    }

    /// Returns true when `other` is a parent of this `FullMarkerId`.
    pub fn within(&self, other: &FullMarkerId) -> bool {
        self.pack_id == other.pack_id && self.marker_id.within(&other.marker_id)
    }
}

impl std::fmt::Display for FullMarkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = self.pack_id.fmt(f);
        let _ = ":".fmt(f);
        self.marker_id.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum MarkerKind {
    #[default]
    Category,
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
    pub id: MarkerId,
    pub label: String,
    pub kind: MarkerKind,
    pub depth: usize,
    pub behavior: Option<Behavior>,
    pub poi_tip: Option<String>,
    pub poi_description: Option<String>,
    pub map_ids: HashSet<u32>,
    pub icon_file: Option<Utf8PathBuf<Utf8UnixEncoding>>,
}

impl Marker {
    pub(super) fn new(
        id: impl Into<Cow<'static, str>>,
        label: impl Into<String>,
        kind: MarkerKind,
    ) -> Self {
        Self {
            id: MarkerId(id.into()),
            label: label.into(),
            kind,
            ..Default::default()
        }
    }

    fn copy_from_parent(&mut self, parent: &Marker) {
        self.id = MarkerId(format!("{}.{}", parent.id, self.id).into());
        self.depth = parent.depth + 1;
        self.behavior = self.behavior.or(parent.behavior);
        if self.icon_file.is_none() {
            self.icon_file = parent.icon_file.clone();
        }
    }

    pub fn from_attrs(attrs: Attributes) -> Result<Self> {
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
                "name" => this.id = MarkerId(value.into()),
                "displayname" => this.label = value,
                "isseparator" => match value.to_lowercase().as_str() {
                    "true" | "1" => this.kind = MarkerKind::Separator,
                    _ => {}
                },
                "iconfile" => {
                    let Ok(path) = Utf8WindowsPathBuf::from_str(&value.to_lowercase());
                    this.icon_file = Some(path.with_unix_encoding().to_path_buf());
                }
                _ => {}
            }
        }
        Ok(this)
    }
}

#[derive(Asset, Reflect, Clone, Debug)]
pub struct Route {
    pub map_id: u32,
    pub path: Vec<Vec3>,
    pub texture_file: String,
}

#[derive(Clone, Debug)]
pub struct MarkerPack {
    id: PackId,

    /// Nodes without any parents. Useful for iterating through all
    /// content in the graph.
    pub roots: BTreeSet<NodeIndex>,

    /// Lookup an index by it's string ID
    pub indices: HashMap<MarkerId, NodeIndex>,

    /// Lookup a marker by its index
    markers: HashMap<NodeIndex, Marker>,

    /// Map the relationship between markers
    graph: DiGraph<NodeIndex, ()>,

    /// POIs associated with markers
    pois: HashMap<MarkerId, Vec<Poi>>,

    /// Trails associated with markers
    trails: HashMap<MarkerId, Vec<Route>>,

    icons: HashMap<String, Handle<Image>>,
}

impl MarkerPack {
    fn new(id: PackId) -> Self {
        Self {
            id,
            roots: Default::default(),
            indices: Default::default(),
            markers: Default::default(),
            graph: Default::default(),
            pois: Default::default(),
            trails: Default::default(),
            icons: Default::default(),
        }
    }

    pub fn full_id(&self, id: MarkerId) -> FullMarkerId {
        FullMarkerId {
            pack_id: self.id.clone(),
            marker_id: id.clone(),
        }
    }

    fn index_of(&self, id: &MarkerId) -> Option<NodeIndex> {
        self.indices.get(id).cloned()
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

    pub fn get_image(&self, path: &str) -> Option<Handle<Image>> {
        self.icons.get(path).cloned()
    }

    pub fn get_trails(&self, id: &MarkerId) -> Option<&Vec<Route>> {
        self.trails.get(id)
    }

    pub fn get_images(&self) -> impl Iterator<Item = &Handle<Image>> {
        self.icons.values()
    }

    pub fn get_map_markers<'a>(
        &'a self,
        map_id: &'a u32,
    ) -> impl Iterator<Item = FullMarkerId> + 'a {
        self.roots()
            .map(|root| {
                self.iter_recursive(&root.id)
                    .filter(|marker| self.contains_map_id(&marker.id, *map_id))
                    .map(|marker| &marker.id)
            })
            .flatten()
            .map(|marker_id| FullMarkerId {
                pack_id: self.id.clone(),
                marker_id: marker_id.clone(),
            })
    }

    pub fn get(&self, id: &MarkerId) -> Option<&Marker> {
        let node_id = self.indices.get(id)?;
        self.markers.get(node_id)
    }

    pub fn get_mut(&mut self, id: &MarkerId) -> Option<&mut Marker> {
        if let Some(node_id) = self.indices.get(id) {
            self.markers.get_mut(node_id)
        } else {
            None
        }
    }

    pub fn roots(&self) -> impl Iterator<Item = &Marker> {
        self.roots
            .iter()
            .filter_map(|index| self.markers.get(index))
    }

    pub fn iter(&self, start: &MarkerId) -> impl Iterator<Item = &Marker> {
        let start_id = self.index_of(start).unwrap();
        self.graph
            .neighbors_directed(start_id, Direction::Outgoing)
            .filter_map(|id| self.markers.get(&id))
    }

    pub fn iter_recursive(&self, start: &MarkerId) -> impl Iterator<Item = &Marker> {
        let start_marker = self.get(start).unwrap();
        [start_marker].into_iter().chain(
            self.iter(start)
                .map(|marker| [marker].into_iter().chain(self.iter(&marker.id)))
                .flatten(),
        )
    }
}

pub struct MarkerPackBuilder {
    tree: MarkerPack,

    edges: HashMap<NodeIndex, BTreeSet<NodeIndex>>,

    trail_tags: HashMap<MarkerId, Vec<TrailXml>>,
    trail_data: HashMap<String, TrailData>,

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
    pub fn new(pack_id: impl Into<PackId>) -> Self {
        Self {
            tree: MarkerPack::new(pack_id.into()),
            edges: Default::default(),
            count: Default::default(),
            parent_id: Default::default(),
            trail_tags: Default::default(),
            trail_data: Default::default(),
        }
    }

    pub fn add_marker(&mut self, mut marker: Marker) -> &mut Self {
        let node_id = if let Some(parent_id) = self.parent_id.front().copied() {
            let parent_marker = self.tree.markers.get(&parent_id).unwrap();
            marker.copy_from_parent(parent_marker);
            let node_id = self.get_or_create_index(marker.id.clone());
            if let Some(children) = self.edges.get_mut(&parent_id) {
                children.insert(node_id);
            } else {
                let mut set = BTreeSet::default();
                set.insert(node_id);
                self.edges.insert(parent_id, set);
            }
            node_id
        } else {
            let node_id = self.get_or_create_index(marker.id.clone());
            self.tree.roots.insert(node_id);
            node_id
        };

        self.parent_id.push_front(node_id);
        self.tree.markers.insert(node_id, marker.clone());
        self
    }

    pub fn add_poi(&mut self, poi: Poi) {
        if let Some(pois) = self.tree.pois.get_mut(&poi.id) {
            pois.push(poi);
        } else {
            self.tree.pois.insert(poi.id.clone(), vec![poi]);
        }
    }

    pub fn add_trail_tag(&mut self, id: MarkerId, trail: TrailXml) {
        if let Some(tags) = self.trail_tags.get_mut(&id) {
            tags.push(trail);
        } else {
            self.trail_tags.insert(id, vec![trail]);
        }
    }

    pub fn add_trail_data(&mut self, file_path: String, data: TrailData) {
        trace!(
            "Found trail data: {pack_id}/{file_path}",
            pack_id = self.tree.id
        );
        if self.trail_data.insert(file_path.clone(), data).is_some() {
            warn!("{file_path} already exists!");
        }
    }

    pub fn add_image(&mut self, file_path: String, image: Image, image_assets: &mut Assets<Image>) {
        // debug!("Found image: {pack_id}/{file_path}", pack_id = self.tree.id);
        let handle = image_assets.add(image);
        self.tree.icons.insert(file_path, handle);
    }

    pub fn add_map_id(&mut self, id: &MarkerId, map_id: u32) {
        if let Some(marker) = self.tree.get_mut(id) {
            marker.map_ids.insert(map_id);
        }
    }

    fn get_or_create_index(&mut self, marker_id: MarkerId) -> NodeIndex {
        self.tree.index_of(&marker_id).unwrap_or_else(|| {
            let node_id = NodeIndex::new({
                let i = self.count;
                self.count += 1;
                i
            });
            self.tree.indices.insert(marker_id, node_id);
            node_id
        })
    }

    pub fn up(&mut self) {
        self.parent_id.pop_front();
    }

    pub fn new_root(&mut self) {
        self.parent_id.clear();
    }

    pub fn build(mut self) -> MarkerPack {
        for node_id in self.tree.markers.keys() {
            self.tree.graph.add_node(*node_id);
        }

        for (parent, children) in self.edges.drain() {
            // Reverse the list of child nodes to ensure they're order
            // is maintained from the marker pack.
            for child in children.into_iter().rev() {
                self.tree.graph.add_edge(parent, child, ());
            }
        }

        for (id, tags) in self.trail_tags.drain() {
            for tag in tags {
                let Some(data) = self.trail_data.remove(&tag.trail_file) else {
                    warn!("TrailData not found for XML tag {id}");
                    continue;
                };
                let route = Route {
                    map_id: data.map_id,
                    path: data.path,
                    texture_file: tag.texture_file,
                };
                if let Some(routes) = self.tree.trails.get_mut(&id) {
                    routes.push(route);
                } else {
                    self.tree.trails.insert(id.clone(), vec![route]);
                }
            }
        }

        self.tree
    }
}
