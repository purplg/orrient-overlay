pub(crate) mod model;
pub mod pack;
mod trail;
mod tree;

use model::MarkerXml;
use orrient_core::prelude::AppState;

use pack::FullMarkerId;
use pack::MarkerId;
use pack::MarkerName;
use pack::MarkerPack;
use pack::MarkerPackBuilder;

use bevy::prelude::*;

use bevy::log::debug;
use bevy::log::warn;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::CompressedImageFormats;
use bevy::render::texture::ImageAddressMode;
use bevy::render::texture::ImageSampler;
use bevy::render::texture::ImageSamplerDescriptor;
use bevy::render::texture::ImageType;
use bevy::utils::HashMap;

use anyhow::Context;
use anyhow::Result;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use crate::events::ReloadMarkersEvent;

#[derive(Resource, Deref)]
struct ConfigDir(PathBuf);

fn load_system(
    mut commands: Commands,
    config_dir: Res<ConfigDir>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("Loading marker packs...");
    match load(config_dir.as_path(), &mut images) {
        Ok(packs) => {
            commands.insert_resource(MarkerPacks(packs));
        }
        Err(err) => {
            warn!("Error loading marker packs {err:?}");
        }
    }
}

fn finish_system(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::WaitingForMumbleLink);
}

fn load(path: &Path, images: &mut Assets<Image>) -> Result<HashMap<PackId, MarkerPack>> {
    let mut packs: HashMap<PackId, MarkerPack> = Default::default();

    let iter = std::fs::read_dir(path).or_else(|_| {
        std::fs::create_dir_all(path)?;
        std::fs::read_dir(path)
    })?;

    for path in iter
        .filter_map(|file| file.ok().map(|file| file.path()))
        .filter(|file| file.is_file())
    {
        let Some(filename) = path
            .file_name()
            .map(|filename| filename.to_string_lossy().to_string())
        else {
            continue;
        };

        let Some(extension) = path.extension().map(|ext| ext.to_string_lossy()) else {
            continue;
        };

        match extension {
            Cow::Borrowed("taco") | Cow::Borrowed("zip") => match read_marker_pack(&path, images) {
                Ok(pack) => {
                    packs.insert(PackId(filename), pack);
                }
                Err(err) => {
                    warn!("Error when reading marker pack {err:?}");
                }
            },
            _ => {
                warn!("Unknown file extension: {:?}", path);
            }
        }
    }
    info!("Finished loading {} pack(s)", packs.len());
    Ok(packs)
}

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackId(pub String);

impl PackId {
    pub fn with_marker(&self, marker_id: MarkerId, marker_name: MarkerName) -> FullMarkerId {
        FullMarkerId {
            pack_id: self.clone(),
            marker_id,
            marker_name,
        }
    }
}

impl std::fmt::Display for PackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Resource, Deref, Debug)]
pub struct MarkerPacks(HashMap<PackId, MarkerPack>);

impl MarkerPacks {
    pub fn get_map_markers<'a>(&'a self, map_id: &'a u32) -> impl Iterator<Item = FullMarkerId> {
        // self.values().flat_map(|pack| pack.get_map_markers(map_id))
        [].into_iter()
    }
}

#[derive(Debug)]
enum Tag {
    OverlayData,
    Marker(model::MarkerXml),
    POIs,
    Poi(model::PoiXml),
    Trail(model::TrailXml),
    UnknownField(String),
}

impl Tag {
    fn from_element(builder: &MarkerPackBuilder, element: &BytesStart) -> Result<Tag> {
        let tag = core::str::from_utf8(element.name().0)?;
        Ok(match tag.to_lowercase().as_ref() {
            "overlaydata" => Tag::OverlayData,
            "markercategory" => Tag::Marker(MarkerXml::from_attrs(element.attributes())?),
            "pois" => Tag::POIs,
            "poi" => Tag::Poi(model::PoiXml::from_attrs(builder, element.attributes())?),
            "trail" => Tag::Trail(model::TrailXml::from_attrs(builder, element.attributes())?),
            field => Tag::UnknownField(field.to_string()),
        })
    }

    fn apply(self, builder: &mut MarkerPackBuilder) {
        match self {
            Tag::OverlayData => {
                builder.new_root();
            }
            Tag::Marker(marker) => {
                builder.add_marker(marker);
            }
            Tag::POIs => {}
            Tag::Poi(poi) => {
                builder.add_poi(poi);
            }
            Tag::Trail(trail) => {
                builder.add_trail_tag(trail);
            }
            Tag::UnknownField(element) => {
                warn!("Unknown Field: {element}");
            }
        }
    }
}

fn read_marker_pack(path: &Path, images: &mut Assets<Image>) -> Result<MarkerPack> {
    let pack_filename = path
        .file_name()
        .context("Could not determine filename in {path:?}")?
        .to_string_lossy()
        .to_string();

    let mut builder = MarkerPackBuilder::new(PackId(pack_filename));
    info!("Parsing: {}", builder.id());

    let pack = File::open(path)?;
    println!("pack: {:?}", pack);
    let mut zip = zip::ZipArchive::new(pack)?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let file_path = file.name().to_string();
        let Some(ext) = file_path.rsplit(".").next() else {
            continue;
        };
        match ext {
            "xml" => {
                let _ = parse_xml(&mut builder, &file_path, BufReader::new(file));
            }
            "png" => {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                let image: Image = Image::from_buffer(
                    &bytes,
                    ImageType::Extension(ext),
                    CompressedImageFormats::all(),
                    false,
                    ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        ..default()
                    }),
                    RenderAssetUsages::all(), // TODO Maybe only needs to be RENDER_WORLD?
                )
                .unwrap();
                builder.add_image(file_path, image, images);
            }
            "trl" => match trail::read(file) {
                Ok(trail_data) => builder.add_trail_data(file_path, trail_data),
                Err(err) => {
                    warn!("Error parsing trail file: {err}: {file_path}")
                }
            },
            ext => debug!("Skipping unknown extension {ext}"),
        }
    }
    info!("Building: {}", builder.id());
    let marker_pack = builder.build();
    info!("Finished: {}", marker_pack.id);
    Ok(marker_pack)
}

fn parse_xml<R: Read + BufRead>(
    builder: &mut MarkerPackBuilder,
    filename: &str,
    reader: R,
) -> Result<()> {
    let mut reader = Reader::from_reader(reader);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(event) => match event {
                Event::Start(element) => {
                    match Tag::from_element(&builder, &element) {
                        Ok(tag) => tag.apply(builder),
                        Err(err) => {
                            warn!(
                                "Error parsing tag {:?} in file {:?}: {:?}",
                                &element, filename, err
                            );
                            continue;
                        }
                    };
                }
                Event::Empty(element) => {
                    match Tag::from_element(&builder, &element) {
                        Ok(tag) => tag.apply(builder),
                        Err(err) => {
                            warn!(
                                "Error parsing tag {:?} in file {:?}: {:?}",
                                &element, filename, err
                            );
                            continue;
                        }
                    };
                    builder.up();
                }
                Event::End(_) => {
                    builder.up();
                }
                Event::Eof => break,
                Event::Decl(_) => {}
                Event::Comment(_) => {}
                unknown_event => debug!("unknown_event in {filename}: {unknown_event:?}"),
            },
            Err(err) => panic!(
                "Error reading {:?} at position {}: {:?}",
                filename,
                reader.buffer_position(),
                err
            ),
        }
    }

    builder.new_root();
    Ok(())
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConfigDir(
            dirs::config_dir()
                .unwrap()
                .join("orrient")
                .join("markers")
                .to_path_buf(),
        ));

        app.add_systems(
            OnEnter(AppState::ParsingMarkerPacks),
            (load_system, finish_system).chain(),
        );
        app.add_systems(Update, load_system.run_if(on_event::<ReloadMarkersEvent>()));
    }
}

// TODO
// #[cfg(ignore)]
// #[cfg(test)]
// mod tests {
//     use std::io::Write;

//     use lazy_static::lazy_static;
//     use pack::MarkerName;
//     use tempfile::tempdir;
//     use zip::{write::SimpleFileOptions, ZipWriter};

//     use super::*;

//     lazy_static! {
//         static ref TEST_PACKS: HashMap<PackId, MarkerPack> = {
//             let dir = tempdir().unwrap().into_path();
//             let mut writer = ZipWriter::new(File::create_new(dir.join("test.taco")).unwrap());
//             let options =
//                 SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

//             writer.start_file("test_1.xml", options).unwrap();
//             writer
//                 .write(
//                     r#"
// <OverlayData>
//   <MarkerCategory name="A" DisplayName="Item A">
//     <MarkerCategory name="B" DisplayName="Item A.B">
//       <MarkerCategory name="C" DisplayName="Item A.B" />
//       <MarkerCategory name="D" DisplayName="Item A.B" />
//     </MarkerCategory>
//   </MarkerCategory>
//   <POIs>
//     <POI MapID="15" xpos="100.0" ypos="100.0" zpos="-100.0" type="A" GUID="none"/>
//     <POI MapID="15" xpos="200.0" ypos="200.0" zpos="-200.0" type="A.B" GUID="none"/>
//   </POIs>
// </OverlayData>
// "#
//                     .as_bytes(),
//                 )
//                 .unwrap();

//             writer.start_file("test_2.xml", options).unwrap();
//             writer
//                 .write(
//                     r#"
// <OverlayData>
//   <MarkerCategory name="A" DisplayName="Item A">
//     <MarkerCategory name="E" DisplayName="Item A.E">
//       <MarkerCategory name="F" DisplayName="Item A.E.F" />
//     </MarkerCategory>
//   </MarkerCategory>
//   <POIs>
//     <POI MapID="15" xpos="300.0" ypos="300.0" zpos="-300.0" type="A.E" GUID="none"/>
//     <POI MapID="15" xpos="400.0" ypos="400.0" zpos="-400.0" type="A.E" GUID="none"/>
//   </POIs>
// </OverlayData>
// "#
//                     .as_bytes(),
//                 )
//                 .unwrap();

//             writer.start_file("test_3.xml", options).unwrap();
//             writer
//                 .write(
//                     r#"
// <OverlayData>
//   <MarkerCategory name="G" DisplayName="Item G">
//     <MarkerCategory name="H" DisplayName="Item G.B" />
//     <MarkerCategory name="I" DisplayName="Item G.I">
//       <MarkerCategory name="J" DisplayName="Item G.I.J" />
//       <MarkerCategory name="K" DisplayName="Item G.I.K" />
//       <MarkerCategory name="L" DisplayName="Item G.I.L" />
//     </MarkerCategory>
//   </MarkerCategory>
//   <POIs>
//     <POI MapID="15" xpos="500.0" ypos="500.0" zpos="-500.0" type="G.K" GUID="none"/>
//     <POI MapID="15" xpos="600.0" ypos="600.0" zpos="-600.0" type="G.L" GUID="none"/>
//   </POIs>
// </OverlayData>
// "#
//                     .as_bytes(),
//                 )
//                 .unwrap();

//             writer.finish().unwrap();
//             let mut images: Assets<Image> = Assets::default();
//             load(&dir, &mut images).unwrap()
//         };
//     }

//     #[test]
//     fn test_iter() {
//         let markers = TEST_PACKS.get(&PackId("test.taco".into())).unwrap();
//         let mut iter = markers.recurse(&MarkerPath::new_root("A".into()));

//         //     A
//         //    / \
//         //   B   E
//         //  / \   \
//         // C   D   F
//         assert_eq!(iter.next().unwrap().id.path.0, "A");
//         assert_eq!(iter.next().unwrap().id.path.0, "A.B");
//         assert_eq!(iter.next().unwrap().id.path.0, "A.B.C");
//         assert_eq!(iter.next().unwrap().id.path.0, "A.B.D");
//         assert_eq!(iter.next().unwrap().id.path.0, "A.E");
//         assert_eq!(iter.next().unwrap().id.path.0, "A.E.F");
//         assert!(iter.next().is_none());

//         //   G
//         //  / \
//         // H   I
//         //   / | \
//         //  J  K  L
//         let mut iter = markers.recurse(&MarkerName("G".into()));
//         assert_eq!(iter.next().unwrap().id.0, "G");
//         assert_eq!(iter.next().unwrap().id.0, "G.H");
//         assert_eq!(iter.next().unwrap().id.0, "G.I");
//         assert_eq!(iter.next().unwrap().id.0, "G.I.J");
//         assert_eq!(iter.next().unwrap().id.0, "G.I.K");
//         assert_eq!(iter.next().unwrap().id.0, "G.I.L");
//         assert!(iter.next().is_none());

//         //     A
//         //    / \
//         //   B   E
//         //  / \   \
//         // C   D   F
//         let mut iter = markers.recurse(&MarkerName("A.B".into()));
//         assert_eq!(iter.next().unwrap().id.0, "A.B");
//         assert_eq!(iter.next().unwrap().id.0, "A.B.C");
//         assert_eq!(iter.next().unwrap().id.0, "A.B.D");
//         assert!(iter.next().is_none());

//         //     A
//         //    / \
//         //   B   E
//         //  / \   \
//         // C   D   F
//         let mut iter = markers.recurse(&MarkerName("A.B.C".into()));
//         assert_eq!(iter.next().unwrap().id.0, "A.B.C");
//         assert!(iter.next().is_none());
//     }

//     #[test]
//     fn test_xml() {
//         let pack = TEST_PACKS.get(&PackId("test.taco".into())).unwrap();
//         let mut roots = pack.roots();
//         let root = roots.next().unwrap();
//         assert_eq!(root.name, MarkerName("A".into()));
//         {
//             let mut iter = pack.iter(&root.name);
//             assert_eq!(iter.next().unwrap().id, MarkerName("A.B".into()));
//             assert_eq!(iter.next().unwrap().id, MarkerName("A.E".into()));
//         }

//         let root = roots.next().unwrap();
//         assert_eq!(root.name, MarkerName("G".into()));
//         {
//             let mut iter = pack.iter(&root.name);
//             assert_eq!(iter.next().unwrap().id, MarkerName("G.H".into()));
//             assert_eq!(iter.next().unwrap().id, MarkerName("G.I".into()));
//         }
//     }

//     #[test]
//     fn test_poi() {
//         let pack = TEST_PACKS.get(&PackId("test.taco".into())).unwrap();

//         {
//             let mut pois = pack
//                 .get_pois(&MarkerPath::new_from_str("A"))
//                 .unwrap()
//                 .iter();
//             let poi: &model::Poi = pois.next().unwrap();
//             assert_eq!(poi.map_id, Some(15));
//             assert_eq!(poi.position.unwrap().x, 100.0);
//             assert_eq!(poi.position.unwrap().y, 100.0);
//             assert_eq!(poi.position.unwrap().z, -100.0);
//         }
//         {
//             let mut pois = pack
//                 .get_pois(&MarkerPath::new_from_str("A.B"))
//                 .unwrap()
//                 .iter();
//             let poi: &model::Poi = pois.next().unwrap();
//             assert_eq!(poi.map_id, Some(15));
//             assert_eq!(poi.position.unwrap().x, 200.0);
//             assert_eq!(poi.position.unwrap().y, 200.0);
//             assert_eq!(poi.position.unwrap().z, -200.0);
//         }
//         {
//             let mut pois = pack
//                 .get_pois(&MarkerPath::new_from_str("A.E"))
//                 .unwrap()
//                 .iter();
//             let poi: &model::Poi = pois.next().unwrap();
//             assert_eq!(poi.map_id, Some(15));
//             assert_eq!(poi.position.unwrap().x, 300.0);
//             assert_eq!(poi.position.unwrap().y, 300.0);
//             assert_eq!(poi.position.unwrap().z, -300.0);

//             let poi: &model::Poi = pois.next().unwrap();
//             assert_eq!(poi.map_id, Some(15));
//             assert_eq!(poi.position.unwrap().x, 400.0);
//             assert_eq!(poi.position.unwrap().y, 400.0);
//             assert_eq!(poi.position.unwrap().z, -400.0);
//         }

//         {
//             let mut pois = pack
//                 .get_pois(&MarkerPath::new_from_str("G.K"))
//                 .unwrap()
//                 .iter();
//             let poi: &model::Poi = pois.next().unwrap();
//             assert_eq!(poi.map_id, Some(15));
//             assert_eq!(poi.position.unwrap().x, 500.0);
//             assert_eq!(poi.position.unwrap().y, 500.0);
//             assert_eq!(poi.position.unwrap().z, -500.0);
//         }

//         {
//             let mut pois = pack
//                 .get_pois(&MarkerPath::new_from_str("G.L"))
//                 .unwrap()
//                 .iter();
//             let poi: &model::Poi = pois.next().unwrap();
//             assert_eq!(poi.map_id, Some(15));
//             assert_eq!(poi.position.unwrap().x, 600.0);
//             assert_eq!(poi.position.unwrap().y, 600.0);
//             assert_eq!(poi.position.unwrap().z, -600.0);
//         }
//     }
// }
