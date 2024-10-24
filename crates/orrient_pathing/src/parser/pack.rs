use bevy::utils::HashMap;
use bevy::{prelude::*, utils::HashSet};
use serde::{Deserialize, Serialize};
use slab_tree::{NodeId, NodeRef, Tree};
use typed_path::Utf8PathBuf;
use typed_path::Utf8UnixEncoding;

use super::model::Behavior;
use super::model::MarkerKind;
use super::model::MarkerXml;
use super::model::PoiXml;
use super::model::TrailXml;
use super::trail::TrailData;
use super::PackId;

#[derive(Clone, Debug)]
pub struct Poi {
    pub map_id: Option<u32>,
    pub position: Option<Vec3>,
    pub icon_file: Option<Utf8PathBuf<Utf8UnixEncoding>>,
}

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkerName(pub Vec<String>);

impl std::fmt::Display for MarkerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.join(".").fmt(f)
    }
}

impl<T, S> From<T> for MarkerName
where
    T: Iterator<Item = S>,
    S: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.map(Into::into).collect::<Vec<_>>())
    }
}

impl MarkerName {
    /// Returns true when `other` is a child of this `MarkerId`.
    pub fn child_of(&self, other: &MarkerName) -> bool {
        self.0.len() != other.0.len() && self.0.starts_with(&other.0)
    }

    /// Returns true when `other` is a parent of this `MarkerId`.
    pub fn parent_of(&self, other: &MarkerName) -> bool {
        self.0.len() != other.0.len() && other.0.starts_with(&self.0)
    }
}

impl MarkerName {
    pub fn from_string(tree: &Tree<Marker>, id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        let mut parts = id.split(".");
        let current_part = parts.next()?;
        let mut path: Vec<String> = vec![];
        let mut current_idx = tree.root().unwrap().children().find_map(|node| {
            if node.data().name == current_part {
                Some(node.node_id())
            } else {
                None
            }
        })?;
        path.push(current_part.to_string());
        for part in parts {
            current_idx = tree.get(current_idx).unwrap().children().find_map(|node| {
                if node.data().name == part {
                    Some(node.node_id())
                } else {
                    None
                }
            })?;
            path.push(current_part.to_string());
        }
        Some(Self(path))
    }
}

#[derive(Hash, Clone, Default, Debug, Deref, PartialEq, Eq)]
pub struct MarkerPath(pub Vec<NodeId>);

impl MarkerPath {
    pub fn from_string(tree: &Tree<Marker>, id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        let mut parts = id.split(".");
        let current_part = parts.next()?;
        let mut path = vec![];
        let mut current_idx = tree.root().unwrap().children().find_map(|node| {
            if node.data().name == current_part {
                Some(node.node_id())
            } else {
                None
            }
        })?;
        path.push(current_idx);
        for part in parts {
            current_idx = tree.get(current_idx).unwrap().children().find_map(|node| {
                if node.data().name == part {
                    Some(node.node_id())
                } else {
                    None
                }
            })?;
            path.push(current_idx);
        }
        Some(Self(path))
    }
}

impl std::fmt::Display for MarkerPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|idx| format!("{:?}", idx))
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[derive(Hash, Copy, Clone, Debug, PartialEq, Eq)]
pub struct MarkerId(pub NodeId);

impl std::ops::Deref for MarkerId {
    type Target = NodeId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NodeId> for MarkerId {
    fn from(value: NodeId) -> MarkerId {
        MarkerId(value)
    }
}

impl From<MarkerId> for NodeId {
    fn from(value: MarkerId) -> NodeId {
        value.0
    }
}

#[derive(Component, Hash, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullMarkerId {
    pub pack_id: PackId,
    pub marker_name: MarkerName,
}

impl FullMarkerId {
    pub fn parent_of(&self, other: &Self) -> bool {
        self.pack_id == other.pack_id && self.marker_name.parent_of(&other.marker_name)
    }

    pub fn child_of(&self, other: &Self) -> bool {
        self.pack_id == other.pack_id && self.marker_name.child_of(&other.marker_name)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Marker {
    pub name: String,
    pub label: String,
    pub kind: MarkerKind,
    pub depth: usize,
    pub behavior: Option<Behavior>,
    pub poi_tip: Option<String>,
    pub poi_description: Option<String>,
    pub map_ids: HashSet<u32>,
    pub icon_file: Option<Utf8PathBuf<Utf8UnixEncoding>>,
    pub texture: Option<String>,

    /// Associated trails
    pub trails: Vec<Route>,

    /// Associated POIs
    pub pois: Vec<Poi>,
}

impl Marker {
    pub fn merge(&mut self, other: Marker) {
        if self.behavior.is_none() {
            self.behavior = other.behavior;
        }
        if self.poi_tip.is_none() {
            self.poi_tip = other.poi_tip;
        }
        if self.poi_description.is_none() {
            self.poi_description = other.poi_description;
        }
        for map_id in other.map_ids.into_iter() {
            self.map_ids.insert(map_id);
        }
        if self.icon_file.is_none() {
            self.icon_file = other.icon_file;
        }
        if self.texture.is_none() {
            self.texture = other.texture;
        }
    }
}

#[derive(Asset, Reflect, Clone, Debug)]
pub struct Route {
    pub map_id: u32,
    pub path: Vec<Vec3>,
    pub texture_file: String,
}

#[derive(Debug)]
pub struct MarkerPack {
    pub tree: Tree<Marker>,

    /// path->icon references
    icons: HashMap<String, Handle<Image>>,
}

impl std::ops::Deref for MarkerPack {
    type Target = Tree<Marker>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

impl std::ops::DerefMut for MarkerPack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tree
    }
}

impl MarkerPack {
    fn new(tree: Tree<Marker>) -> Self {
        Self {
            tree,
            icons: Default::default(),
        }
    }

    pub fn id(&self) -> &str {
        &self.tree.root().unwrap().data().name
    }

    pub fn roots(&self) -> impl Iterator<Item = NodeId> + use<'_> {
        self.tree
            .root()
            .unwrap()
            .children()
            .map(|node| node.node_id())
    }

    pub fn iter(&self, id: impl Into<NodeId>) -> impl Iterator<Item = NodeRef<'_, Marker>> {
        self.tree.get(id.into()).unwrap().children()
    }

    pub fn recurse(&self, id: impl Into<NodeId>) -> impl Iterator<Item = NodeRef<'_, Marker>> {
        self.tree.get(id.into()).unwrap().traverse_pre_order()
    }

    pub fn full_id(&self, id: impl Into<NodeId>) -> FullMarkerId {
        FullMarkerId {
            pack_id: PackId(self.id().to_owned()),
            marker_name: self.name_of(id),
        }
    }

    pub fn name_of(&self, id: impl Into<NodeId>) -> MarkerName {
        let node = self.tree.get(id.into()).unwrap();
        let mut name = node
            .ancestors()
            .map(|node| node.data().name.clone())
            .collect::<Vec<_>>();
        name.pop();
        name.reverse();
        name.push(node.data().name.clone());
        MarkerName(name)
    }

    pub fn contains_map_id(&self, id: impl Into<NodeId>, map_id: u32) -> bool {
        for node in self.recurse(id.into()) {
            if node.data().map_ids.contains(&map_id) {
                return true;
            }
        }
        false
    }

    pub fn get_image(&self, path: &str) -> Option<Handle<Image>> {
        self.icons.get(path).cloned()
    }

    pub fn get_images(&self) -> impl Iterator<Item = &Handle<Image>> {
        self.icons.values()
    }

    pub fn find_by_name<'a>(&self, name: impl Into<MarkerName>) -> Option<NodeId> {
        let name = name.into();
        for node in self.root().unwrap().traverse_pre_order() {
            if name == self.name_of(node.node_id()) {
                return Some(node.node_id());
            }
        }
        None
    }
}

pub struct MarkerPackBuilder {
    pub pack: MarkerPack,
    parents: Vec<NodeId>,

    /// Store the found poi tags to be handler after all markers have
    /// been found.
    poi_tags: Vec<PoiXml>,

    /// Store the found trail tags to be handler after all markers
    /// have been found.
    trail_tags: Vec<TrailXml>,

    /// Store the found trail files to be handler after all markers
    /// have been found.
    trail_data: HashMap<String, TrailData>,
}

impl MarkerPackBuilder {
    pub fn new(pack_id: String) -> Self {
        let mut tree = Tree::new();
        let root_id = tree.set_root(Marker {
            name: pack_id,
            ..default()
        });
        Self {
            pack: MarkerPack::new(tree),
            parents: vec![root_id],
            poi_tags: Default::default(),
            trail_tags: Default::default(),
            trail_data: Default::default(),
        }
    }

    pub fn add_poi(&mut self, poi: PoiXml) {
        self.poi_tags.push(poi);
    }

    pub fn add_trail_tag(&mut self, trail: TrailXml) {
        self.trail_tags.push(trail);
    }

    pub fn add_trail_data(&mut self, file_path: String, data: TrailData) {
        if self.trail_data.insert(file_path.clone(), data).is_some() {
            warn!("{file_path} already exists!");
        }
    }

    pub fn add_image(&mut self, file_path: String, image: Image, image_assets: &mut Assets<Image>) {
        // debug!("Found image: {pack_id}/{file_path}", pack_id = self.tree.id);
        let handle = image_assets.add(image);
        // TODO
        // self.tree.icons.insert(file_path, handle);
    }

    pub fn add_marker(&mut self, xml: MarkerXml) -> NodeId {
        let parent_id = self.parents.last().unwrap();
        let id = if let Some(node) = self
            .pack
            .get(*parent_id)
            .unwrap()
            .children()
            .find(|node| node.data().name == xml.name)
        {
            node.node_id()
        } else {
            self.pack
                .get_mut(*parent_id)
                .unwrap()
                .append(Marker {
                    name: xml.name,
                    label: xml.label,
                    kind: xml.kind,
                    depth: xml.depth,
                    behavior: xml.behavior,
                    poi_tip: xml.poi_tip,
                    poi_description: xml.poi_description,
                    map_ids: xml.map_ids,
                    icon_file: xml.icon_file,
                    texture: xml.texture,
                    trails: vec![],
                    pois: vec![],
                })
                .node_id()
        };
        self.parents.push(id);
        id
    }

    pub fn new_root(&mut self) {
        self.parents.clear();
        self.parents.push(self.pack.root_id().unwrap())
    }

    pub fn up(&mut self) {
        if self.parents.len() > 1 {
            self.parents.pop();
        }
    }

    pub fn build(mut self) -> MarkerPack {
        let pack_id = self.pack.id().to_owned();

        // Attach POI's
        let pois = self.poi_tags.drain(..).collect::<Vec<_>>();
        for poi in pois {
            let Some(mut node) = self
                .pack
                .find_by_name(poi.id.split("."))
                .and_then(|node_id| self.pack.get_mut(node_id))
            else {
                warn!("Could not find Marker for Poi id: {}", poi.id);
                continue;
            };
            let marker = node.data();

            if let Some(map_id) = poi.map_id {
                marker.map_ids.insert(map_id);
            }

            marker.pois.push(Poi {
                map_id: poi.map_id,
                position: poi.position,
                icon_file: poi.icon_file,
            });
        }

        // Attach trail tags
        let trails = self.trail_tags.drain(..).collect::<Vec<_>>();
        for trail in trails {
            let Some(data) = self.trail_data.get(&trail.trail_file) else {
                warn!(
                    "Associated TrailData not found for {:?}:{:?}: {}",
                    pack_id, trail.id, trail.trail_file
                );
                continue;
            };

            let Some(mut node) = self
                .pack
                .find_by_name(trail.id.split("."))
                .and_then(|node_id| self.pack.get_mut(node_id))
            else {
                warn!("Could not find Marker for Trail id: {}", trail.id);
                continue;
            };
            let marker = node.data();

            marker.map_ids.insert(data.map_id);

            let Some(texture) = trail
                .texture_file
                .as_ref()
                .or_else(|| marker.texture.as_ref())
            else {
                warn!("Trail has no texture: {}", trail.id);
                continue;
            };

            let route = Route {
                map_id: data.map_id,
                path: data.path.clone(),
                texture_file: texture.to_string(),
            };
            marker.trails.push(route);
        }

        self.pack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(name: &str) -> MarkerXml {
        MarkerXml {
            name: name.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_path_from_string() {
        let mut builder = MarkerPackBuilder::new("pack".to_string());
        let n_1 = &builder.add_marker(marker("one"));
        let _n_2 = &builder.add_marker(marker("two"));
        builder.up();
        let n_3 = &builder.add_marker(marker("three"));
        let n_4 = &builder.add_marker(marker("four"));
        assert_eq!(
            MarkerPath::from_string(&builder.pack, "one.three.four").unwrap(),
            MarkerPath(vec![*n_1, *n_3, *n_4])
        );
    }

    fn id(items: impl IntoIterator<Item = impl ToString>) -> FullMarkerId {
        FullMarkerId {
            pack_id: PackId("pack".to_string()),
            marker_name: MarkerName(items.into_iter().map(|i| i.to_string()).collect::<Vec<_>>()),
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_parent_of() {
        assert!( id(["a", "b"     ]).parent_of(&id(["a", "b", "c"])));
        assert!(!id(["a", "b", "c"]).parent_of(&id(["a", "b"     ])));
        assert!(!id(["a", "b", "c"]).parent_of(&id(["a", "b", "c"])));
        assert!(!id(["a", "b"     ]).parent_of(&id(["a", "d", "c"])));
    }

    #[test]
    #[rustfmt::skip]
    fn test_child_of() {
        assert!( id(["a", "b", "c"]).child_of(&id(["a", "b"     ])));
        assert!(!id(["a", "b"     ]).child_of(&id(["a", "b", "c"])));
        assert!(!id(["a", "b", "c"]).child_of(&id(["a", "b", "c"])));
        assert!(!id(["a", "b", "c"]).child_of(&id(["a", "d"     ])));
    }
}
