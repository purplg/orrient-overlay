use bevy::utils::HashMap;
use bevy::{prelude::*, utils::HashSet};
use itertools::Itertools;
use petgraph::prelude::*;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use typed_path::Utf8PathBuf;
use typed_path::Utf8UnixEncoding;

use super::model::Behavior;
use super::model::MarkerKind;
use super::model::MarkerXml;
use super::model::PoiXml;
use super::model::TrailXml;
use super::trail::TrailData;
use super::tree::TreeBuilder;
use super::tree::Trees;
use super::PackId;

#[derive(Clone, Debug)]
pub struct Poi {
    pub marker_id: MarkerId,
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
    pub fn from_string(builder: &Trees<Marker>, id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        let mut parts = id.split(".");
        let current_part = parts.next()?;
        let mut path: Vec<String> = vec![];
        let mut current_idx = *builder
            .roots
            .iter()
            .filter_map(|idx| builder.get(*idx).map(|marker| (idx, marker)))
            .find_map(|(idx, marker)| {
                if marker.name == current_part {
                    Some(idx)
                } else {
                    None
                }
            })?;
        path.push(current_part.to_string());
        for part in parts {
            current_idx = builder
                .graph()
                .neighbors_directed(current_idx, Direction::Outgoing)
                .filter_map(|idx| builder.get(idx).map(|marker| (idx, marker)))
                .find_map(
                    |(idx, marker)| {
                        if marker.name == part {
                            Some(idx)
                        } else {
                            None
                        }
                    },
                )?;
            path.push(current_part.to_string());
        }
        Some(Self(path))
    }
}

#[derive(Hash, Clone, Default, Debug, Deref, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkerPath(pub Vec<NodeIndex>);

impl MarkerPath {
    pub fn from_string(builder: &Trees<Marker>, id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        let mut parts = id.split(".");
        let current_part = parts.next()?;
        let mut path = vec![];
        let mut current_idx = *builder
            .roots
            .iter()
            .filter_map(|idx| builder.get(*idx).map(|marker| (idx, marker)))
            .find_map(|(idx, marker)| {
                if marker.name == current_part {
                    Some(idx)
                } else {
                    None
                }
            })?;
        path.push(current_idx);
        for part in parts {
            current_idx = builder
                .graph()
                .neighbors_directed(current_idx, Direction::Outgoing)
                .filter_map(|idx| builder.get(idx).map(|marker| (idx, marker)))
                .find_map(
                    |(idx, marker)| {
                        if marker.name == part {
                            Some(idx)
                        } else {
                            None
                        }
                    },
                )?;
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

#[derive(Hash, Copy, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkerId(pub NodeIndex);

impl From<NodeIndex> for MarkerId {
    fn from(value: NodeIndex) -> MarkerId {
        MarkerId(value)
    }
}

impl From<usize> for MarkerId {
    fn from(value: usize) -> MarkerId {
        MarkerId(NodeIndex::new(value))
    }
}

impl std::fmt::Display for MarkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.index().fmt(f)
    }
}

#[derive(Component, Hash, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullMarkerId {
    pub pack_id: PackId,
    pub marker_id: MarkerId,
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
    pub path: MarkerPath,
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

#[derive(Asset, Reflect, Clone, Debug)]
pub struct Route {
    pub map_id: u32,
    pub path: Vec<Vec3>,
    pub texture_file: String,
}

#[derive(Debug)]
pub struct MarkerPack {
    pub id: PackId,

    pub trees: Trees<Marker>,

    /// path->icon references
    icons: HashMap<String, Handle<Image>>,
}

impl MarkerPack {
    fn new(id: PackId, trees: Trees<Marker>) -> Self {
        Self {
            id,
            trees,
            icons: Default::default(),
        }
    }

    pub fn roots(&self) -> impl Iterator<Item = MarkerId> + use<'_> {
        self.trees.roots.iter().map(|idx| (*idx).into())
    }

    pub fn iter(&self, id: MarkerId) -> impl Iterator<Item = (MarkerId, &Marker)> {
        self.trees
            .children(id.0)
            .map(|(idx, marker)| (MarkerId(idx), marker))
    }

    pub fn recurse(&self, id: MarkerId) -> impl Iterator<Item = (MarkerId, &Marker)> {
        self.trees
            .recurse(id.0)
            .map(|(idx, marker)| (MarkerId(idx), marker))
    }

    pub fn full_id(&self, id: MarkerId) -> FullMarkerId {
        FullMarkerId {
            pack_id: self.id.clone(),
            marker_id: id,
            marker_name: self.name_of(id),
        }
    }

    pub fn name_of(&self, marker_id: MarkerId) -> MarkerName {
        MarkerName(
            self.trees
                .path_to(marker_id.0)
                .into_iter()
                .filter_map(|idx| self.trees.get(idx).map(|marker| marker.name.clone()))
                .rev()
                .collect::<Vec<_>>(),
        )
    }

    pub fn contains_map_id(&self, id: MarkerId, map_id: u32) -> bool {
        for (_id, marker) in self.recurse(id) {
            if marker.map_ids.contains(&map_id) {
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

    pub fn get_marker(&self, id: impl Into<MarkerId>) -> Option<&Marker> {
        self.trees.get(id.into().0)
    }
}

pub struct MarkerPackBuilder {
    pack_id: PackId,
    pub tree: TreeBuilder<Marker>,

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
    pub fn new(pack_id: impl Into<PackId>) -> Self {
        Self {
            pack_id: pack_id.into(),
            tree: TreeBuilder::new(),
            poi_tags: Default::default(),
            trail_tags: Default::default(),
            trail_data: Default::default(),
        }
    }

    pub fn id(&self) -> &PackId {
        &self.pack_id
    }

    pub fn find_mut<'a>(&'a mut self, path: &'a MarkerPath) -> Option<(MarkerId, &'a mut Marker)> {
        self.tree
            .find_mut(&path.0)
            .map(|(idx, marker)| (MarkerId(idx), marker))
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

    pub fn add_marker(&mut self, xml: MarkerXml) -> NodeIndex {
        self.tree.insert(Marker {
            path: self.tree.path(),
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
    }

    pub fn new_root(&mut self) {
        self.tree.new_root();
    }

    pub fn up(&mut self) {
        self.tree.up();
    }

    pub fn build(mut self) -> MarkerPack {
        let pack_id = self.pack_id.clone();

        // Merge roots that are the same.
        let primary_roots = self
            .tree
            .roots
            .iter()
            .filter_map(|idx| self.tree.get(*idx).map(|marker| (idx, marker)))
            .unique_by(|(_idx, marker)| &marker.name)
            .map(|(idx, _marker)| idx)
            .collect::<Vec<_>>();
        let mut dups: Vec<(NodeIndex, NodeIndex)> = vec![];
        for root in &self.tree.roots {
            if primary_roots.contains(&root) {
                continue;
            }
            let dup_marker = self.tree.get(*root).unwrap();
            let (idx, _) = self
                .tree
                .roots
                .iter()
                .filter_map(|idx| self.tree.get(*idx).map(|marker| (idx, marker)))
                .find(|(_primary_idx, primary_marker)| primary_marker.name == dup_marker.name)
                .unwrap();
            dups.push((*idx, *root));
        }
        for (root_idx, dup_idx) in dups.drain(..) {
            self.tree.merge(root_idx, dup_idx);
        }

        // Attach POI's
        let pois = self.poi_tags.drain(..).collect::<Vec<_>>();
        for poi in pois {
            let Some(path) = MarkerPath::from_string(&self.tree, poi.id.clone()) else {
                warn!("Could not create MarkerPath from {}", poi.id);
                continue;
            };

            let Some((idx, marker)) = self.find_mut(&path) else {
                warn!("Could not find MarkerPath for POI id: {}", poi.id);
                continue;
            };

            if let Some(map_id) = poi.map_id {
                marker.map_ids.insert(map_id);
            }

            marker.pois.push(Poi {
                marker_id: idx,
                map_id: poi.map_id,
                position: poi.position,
                icon_file: poi.icon_file,
            });
        }

        // Attach trail tags
        let trails = self.trail_tags.drain(..).collect::<Vec<_>>();
        for trail in trails {
            let Some(path) = MarkerPath::from_string(&self.tree, trail.id.clone()) else {
                continue;
            };

            let Some(data) = self.trail_data.get(&trail.trail_file) else {
                warn!(
                    "Associated TrailData not found for {:?}:{:?}: {}",
                    pack_id, trail.id, trail.trail_file
                );
                continue;
            };

            if let Some((_marker_idx, marker)) = self.tree.find_mut(&*path) {
                marker.map_ids.insert(data.map_id);
            }

            let Some(texture) = trail.texture_file.as_ref().or_else(|| {
                self.tree
                    .find(&*path)
                    .and_then(|(_id, marker)| marker.texture.as_ref())
            }) else {
                warn!("Trail has no texture: {path}");
                continue;
            };

            let route = Route {
                map_id: data.map_id,
                path: data.path.clone(),
                texture_file: texture.to_string(),
            };

            if let Some((_idx, marker)) = self.find_mut(&path) {
                marker.trails.push(route);
            };
        }

        MarkerPack::new(self.pack_id, self.tree.build())
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
        let mut builder = MarkerPackBuilder::new(PackId("pack".to_string()));
        let n_1 = &builder.add_marker(marker("one"));
        let _n_2 = &builder.add_marker(marker("two"));
        builder.up();
        let n_3 = &builder.add_marker(marker("three"));
        let n_4 = &builder.add_marker(marker("four"));
        assert_eq!(
            MarkerPath::from_string(&builder.tree, "one.three.four").unwrap(),
            MarkerPath(vec![*n_1, *n_3, *n_4])
        );
    }

    fn id(items: impl IntoIterator<Item = impl ToString>) -> FullMarkerId {
        FullMarkerId {
            pack_id: PackId("pack".to_string()),
            marker_id: MarkerId(0.into()),
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
