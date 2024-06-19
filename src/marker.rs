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

        let mut markers = MarkerSet::default();

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
        println!("markers loaded");
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
                    let id = category.name.clone();
                    categories.insert(id, category.into());
                }
                categories
            },
        }
    }
}

#[derive(Resource, Clone, Deref, Debug, Default)]
pub struct MarkerSet(HashMap<String, Category>);

impl MarkerSet {
    fn insert(&mut self, marker: MarkerCategory) {
        let category: Category = marker.into();
        if let Some(existing) = self.0.get_mut(&category.id) {
            existing.merge(category.subcategories);
        } else {
            self.0.insert(category.id.clone(), category);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub subcategories: HashMap<String, Category>,
}

impl Category {
    fn insert(&mut self, category: Category) {
        if let Some(subcat) = self.subcategories.get_mut(&category.id) {
            subcat.merge(category.subcategories)
        } else {
            self.subcategories.insert(category.id.clone(), category);
        }
    }

    fn merge(&mut self, mut subcategories: HashMap<String, Category>) {
        for (_, category) in subcategories.drain() {
            self.insert(category)
        }
    }
}
