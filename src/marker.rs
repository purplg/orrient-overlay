use bevy::{prelude::*, utils::HashMap};
use marker::MarkerCategory;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarkerSet>();
        app.add_systems(Startup, setup);
        app.add_systems(PreUpdate, load_marker.run_if(on_event::<OrrientEvent>()));
    }
}

fn setup(mut events: EventWriter<OrrientEvent>) {
    events.send(OrrientEvent::LoadMarkers("tw_lws03e05_draconismons.xml".into()));
}

fn load_marker(mut markerset: ResMut<MarkerSet>, mut events: EventReader<OrrientEvent>) {
    for event in events.read() {
        if let OrrientEvent::LoadMarkers(filename) = event {
            let marker_path = xdg::BaseDirectories::with_prefix("orrient")
                .unwrap()
                .get_config_home()
                .join("markers")
                .join(filename);

            if let Ok(data) = marker::read(&marker_path) {
                markerset.0.clear();
                for category in data.categories {
                    markerset.insert(category);
                }
            }
        }
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
