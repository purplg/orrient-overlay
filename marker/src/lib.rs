mod marker;
pub mod trail;

use std::{
    collections::{btree_map, BTreeMap},
    path::Path,
};

#[derive(Debug)]
pub enum Error {
    FsErr(std::io::Error),
    DeErr(quick_xml::de::DeError),
}

pub fn read(path: &Path) -> Result<Markers, Error> {
    fn insert(
        data: &marker::OverlayData,
        category: &marker::MarkerCategory,
        depth: u8,
        mut path: Vec<String>,
    ) -> (String, Marker) {
        // If there is no prefix, we don't need to put a `.` separator.
        let id = category.id();
        path.push(id.clone());
        let long_id = path.join(".");
        let mut marker = Marker::new(
            category.display_name(),
            data.pois
                .iter()
                .find_map(|poi| poi.trail.iter().find(|trail| trail.id == long_id))
                .map(|trail| trail.trail_data.clone()),
        );

        // Search through all POIs to collect the ones that matches
        // the ID of the marker
        for pois in &data.pois {
            for poi in pois.poi.iter() {
                if poi.id == long_id {
                    marker.pois.push(POI {
                        x: poi.x,
                        y: poi.y,
                        z: poi.z,
                    });
                }
            }
        }
        println!("marker.pois for {:?}: {:?}", id, marker.pois.len());

        // Repeat this recursively for all subcategories as well.
        for category in &category.categories {
            let (sub_id, sub_marker) = insert(data, category, depth + 1, path.clone());
            marker.markers.insert(sub_id, sub_marker);
        }

        (id, marker)
    }

    let content = std::fs::read_to_string(path).map_err(Error::FsErr)?;
    let data: marker::OverlayData = quick_xml::de::from_str(&content).map_err(Error::DeErr)?;

    let mut markers: BTreeMap<String, Marker> = Default::default();
    for category in &data.categories {
        let (sub_id, sub_marker) = insert(&data, category, 0, vec![]);
        markers.insert(sub_id, sub_marker);
    }

    Ok(Markers(markers))
}

#[derive(Clone, Debug, Default)]
pub struct Markers(BTreeMap<String, Marker>);

impl Markers {
    pub fn iter(&self) -> Iter {
        Iter {
            top: self.0.iter(),
            sub: None,
            path: vec![],
        }
    }

    fn insert(&mut self, id: String, marker: Marker) -> Option<Marker> {
        self.0.insert(id, marker)
    }

    pub fn get_path<'a>(&'a self, mut path: Vec<&'a str>) -> Option<&'a Marker> {
        if path.len() == 0 {
            return None;
        }
        let id = path.remove(0);
        if path.len() == 0 {
            self.0.get(id)
        } else {
            self.0
                .get(id)
                .and_then(|marker| marker.markers.get_path(path))
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Marker {
    pub label: String,
    pub pois: Vec<POI>,
    pub trail_file: Option<String>,
    pub markers: Markers,
}

impl Marker {
    fn new<L: Into<String>>(label: L, trail_file: Option<String>) -> Self {
        Self {
            label: label.into(),
            pois: Default::default(),
            markers: Default::default(),
            trail_file,
        }
    }

    fn iter<'a>(&'a self, path: Vec<String>) -> Iter<'a> {
        Iter {
            top: self.markers.0.iter(),
            sub: None,
            path,
        }
    }
}

pub struct Iter<'a> {
    top: btree_map::Iter<'a, String, Marker>,
    sub: Option<Box<Iter<'a>>>,
    path: Vec<String>,
}

#[derive(Debug)]
pub struct MarkerEntry<'a> {
    pub id: &'a String,
    pub path: Vec<String>,
    pub marker: &'a Marker,
}

impl<'a> Iterator for Iter<'a> {
    type Item = MarkerEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.sub.as_mut().map(Iterator::next).flatten().or_else(|| {
            let next_top = self.top.next();
            self.sub = next_top.map(|(id, marker)| {
                let mut new_path = self.path.clone();
                new_path.push(id.to_string());
                Box::new(marker.iter(new_path))
            });
            next_top.map(|(id, marker)| {
                let mut new_path = self.path.clone();
                new_path.push(id.to_string());
                MarkerEntry {
                    id,
                    path: new_path,
                    marker,
                }
            })
        })
    }
}

#[derive(Clone, Debug)]
pub struct POI {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug)]
pub struct Trail {}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_markers() -> Markers {
        let mut markers = Markers::default();
        markers.0.insert(
            "1".to_string(),
            Marker {
                label: "1 name".to_string(),
                pois: Default::default(),
                trail_file: Default::default(),
                markers: {
                    let mut markers = Markers::default();
                    markers.insert(
                        "1.1".to_string(),
                        Marker {
                            label: "1.1 name".to_string(),
                            pois: Default::default(),
                            trail_file: Default::default(),
                            markers: {
                                let mut markers = Markers::default();
                                markers
                                    .insert("1.1.1".to_string(), Marker::new("1.1.1 name", None));
                                markers
                                    .insert("1.1.2".to_string(), Marker::new("1.1.2 name", None));
                                markers
                            },
                        },
                    );
                    markers.insert(
                        "1.2".to_string(),
                        Marker {
                            label: "1.2 name".to_string(),
                            pois: Default::default(),
                            trail_file: Default::default(),
                            markers: {
                                let mut markers = Markers::default();
                                markers
                                    .insert("1.2.1".to_string(), Marker::new("1.2.1 name", None));
                                markers
                                    .insert("1.2.2".to_string(), Marker::new("1.2.2 name", None));
                                markers
                            },
                        },
                    );
                    markers
                },
            },
        );
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
        let mut iter = markers.iter();
        assert_eq!(iter.next().unwrap().id, "1");
        assert_eq!(iter.next().unwrap().id, "1.1");
        assert_eq!(iter.next().unwrap().id, "1.1.1");
        assert_eq!(iter.next().unwrap().id, "1.1.2");
        assert_eq!(iter.next().unwrap().id, "1.2");
        assert_eq!(iter.next().unwrap().id, "1.2.1");
        assert_eq!(iter.next().unwrap().id, "1.2.2");
    }

    #[test]
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

    #[test]
    fn test_real_get_path() {
        let markers: Markers = read(Path::new(
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
