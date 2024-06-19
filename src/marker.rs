use bevy::{prelude::*, utils::HashMap};
use marker::MarkerCategory;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let marker_path = xdg::BaseDirectories::with_prefix("orrient")
            .unwrap()
            .get_config_home()
            .join("markers");

        let iter = match std::fs::read_dir(&marker_path) {
            Ok(iter) => iter,
            Err(err) => {
                println!(
                    "Error when opening marker directory: '{:?}' {:?}",
                    marker_path, err
                );
                return;
            }
        };

        let mut markers = Markers::default();

        for data in iter
            // List directory contents
            .filter_map(|direntry| direntry.ok().map(|file| file.path()))
            // Only show files
            .filter(|path| path.is_file())
            // Only XML files
            .filter(|file_path| {
                file_path
                    .extension()
                    .map(|ext| ext == "xml")
                    .unwrap_or_default()
            })
            .filter_map(|path| marker::read(&path).ok())
        {
            for category in data.categories {
                markers.insert(category);
            }
        }
        app.insert_resource(markers);
    }
}

impl From<MarkerCategory> for Category {
    fn from(category: MarkerCategory) -> Self {
        Category {
            id: category.name,
            name: category.display_name,
            subcategories: {
                let mut categories = HashMap::<String, Category>::default();
                for category in category.categories {
                    categories.insert(category.name.clone(), category.into());
                }
                categories
            },
        }
    }
}

#[derive(Resource, Clone, Deref, Debug, Default)]
pub struct Markers(HashMap<String, Category>);

impl Markers {
    fn insert(&mut self, marker: MarkerCategory) {
        let category: Category = marker.into();
        self.0.insert(category.id.clone(), category);
    }
}

#[derive(Clone, Debug)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub subcategories: HashMap<String, Category>,
}
