mod model;
mod pack;
mod trail;

pub mod prelude {
    pub use super::pack::Behavior;
    pub use super::pack::FullMarkerId;
    pub use super::pack::MarkerId;
    pub use super::MarkerPacks;
}

use bevy::utils::HashMap;
use bevy_inspector_egui::egui::TextBuffer;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{CompressedImageFormats, ImageSampler, ImageType};
use pack::{FullMarkerId, Marker, MarkerId, MarkerPack, MarkerPackBuilder};

use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::path::PathBuf;

use bevy::log::{debug, warn};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

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

        app.add_systems(Startup, load_system);
    }
}

#[derive(Resource, Deref)]
struct ConfigDir(PathBuf);

fn load_system(
    mut commands: Commands,
    config_dir: Res<ConfigDir>,
    mut images: ResMut<Assets<Image>>,
) {
    match load(config_dir.as_path(), &mut images) {
        Ok(pack) => {
            commands.insert_resource(MarkerPacks(pack));
        }
        Err(err) => {
            warn!("Error loading marker packs {err:?}");
        }
    }
}

fn load(path: &Path, mut images: &mut Assets<Image>) -> Result<HashMap<PackId, MarkerPack>, Error> {
    let mut packs: HashMap<PackId, MarkerPack> = Default::default();

    let iter = std::fs::read_dir(path).unwrap();
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

        match extension.as_str() {
            "taco" | "zip" => match read_marker_pack(&path, &mut images) {
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
    Ok(packs)
}

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq)]
pub struct PackId(pub String);

impl PackId {
    pub fn with_marker_id(&self, marker_id: MarkerId) -> FullMarkerId {
        FullMarkerId {
            pack_id: self.clone(),
            marker_id,
        }
    }
}

impl From<PackId> for String {
    fn from(value: PackId) -> Self {
        value.0
    }
}

impl std::fmt::Display for PackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Resource, Clone, Deref, Debug)]
pub struct MarkerPacks(HashMap<PackId, MarkerPack>);

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

    fn apply(self, builder: &mut MarkerPackBuilder) {
        match self {
            Tag::OverlayData => {
                builder.new_root();
            }
            Tag::Marker(marker) => {
                builder.add_marker(marker);
            }
            Tag::POIs => {}
            Tag::POI(poi) => {
                builder.add_map_id(&poi.id, poi.map_id);
                builder.add_poi(poi);
            }
            Tag::Trail(trail) => {
                // let id: MarkerId = trail.id.into();
                // if let Some(trail) = self.add_trail(trail) {
                //     self.add_map_id(id, trail.map_id);
                // }
            }
            Tag::Route => {}
            Tag::UnknownField(_) => {}
            Tag::CorruptField(_) => todo!(),
        }
    }
}

fn read_marker_pack(path: &Path, mut images: &mut Assets<Image>) -> Result<MarkerPack, Error> {
    let name = path.to_string_lossy().to_string();

    let mut builder = MarkerPackBuilder::new(PackId(name));

    let pack = File::open(path).map_err(Error::IoErr)?;
    let mut zip = zip::ZipArchive::new(pack).map_err(Error::ZipErr)?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(Error::ZipErr)?;
        let filename = file.name().to_string();
        let Some(ext) = filename.rsplit(".").next() else {
            continue;
        };
        match ext {
            "xml" => {
                let _ = parse_xml(&mut builder, &filename, BufReader::new(file));
            }
            "png" => {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).map_err(Error::IoErr)?;
                let image: Image = Image::from_buffer(
                    &bytes,
                    ImageType::Extension(ext),
                    CompressedImageFormats::NONE,
                    false,
                    ImageSampler::Default,
                    RenderAssetUsages::all(),
                )
                .unwrap();
                builder.add_image(filename, image, &mut images);
            }
            _ => (),
        }
    }
    Ok(builder.build())
}

fn parse_xml<R: Read + BufRead>(
    tree: &mut MarkerPackBuilder,
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
                    match Tag::from_element(&element) {
                        Ok(tag) => tag.apply(tree),
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
                    match Tag::from_element(&element) {
                        Ok(tag) => tag.apply(tree),
                        Err(err) => {
                            warn!(
                                "Error parsing tag {:?} in file {:?}: {:?}",
                                &element, filename, err
                            );
                            continue;
                        }
                    };
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

#[cfg(test)]
mod tests {
    use pack::MarkerKind;

    use super::*;

    //     A            G
    //    / \          / \
    //   B   E        H   I
    //  / \   \         / | \
    // C   D   F       J  K  L
    fn fake_markers() -> MarkerPack {
        let mut markers = MarkerPackBuilder::new();
        markers.add_marker(Marker::new("A", "A Name", MarkerKind::Category));
        markers.add_marker(Marker::new("B", "B Name", MarkerKind::Category));
        markers.add_marker(Marker::new("C", "C Name", MarkerKind::Category));
        markers.up();
        markers.add_marker(Marker::new("D", "D Name", MarkerKind::Category));
        markers.up();
        markers.up();
        markers.add_marker(Marker::new("E", "E Name", MarkerKind::Category));
        markers.add_marker(Marker::new("F", "F Name", MarkerKind::Category));

        markers.new_root();
        markers.add_marker(Marker::new("G", "G Name", MarkerKind::Category));
        markers.add_marker(Marker::new("H", "H Name", MarkerKind::Category));
        markers.up();
        markers.add_marker(Marker::new("I", "I Name", MarkerKind::Category));
        markers.add_marker(Marker::new("J", "J Name", MarkerKind::Category));
        markers.up();
        markers.add_marker(Marker::new("K", "K Name", MarkerKind::Category));
        markers.up();
        markers.add_marker(Marker::new("L", "L Name", MarkerKind::Category));
        markers.build()
    }

    #[test]
    fn test_real_data() {
        env_logger::init();

        load(
            &dirs::config_dir().unwrap().join("orrient").join("markers"),
            &mut Assets::default(),
        )
        .unwrap();
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
