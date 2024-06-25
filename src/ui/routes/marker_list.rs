use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::{
    marker::MarkerSet,
    ui::components::{List, ListItem},
};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, build_route.before(UiSystems::Compute));
    }
}

#[derive(Component)]
pub struct MarkerList;

fn build_route(
    mut commands: Commands,
    markers: Option<Res<MarkerSet>>,
    query: Query<Entity, Added<MarkerList>>,
) {
    let Some(markers) = markers else {
        return;
    };

    for entity in &query {
        commands
            .entity(entity)
            .insert((
                Name::new("MainRoute"),
                SpatialBundle::default(), //
            ))
            .with_children(|route| {
                route
                    .spawn((
                        UiTreeBundle::<MainUi>::from(UiTree::new("Main")), //
                        MovableByCamera,
                    ))
                    .with_children(|ui| {
                        let root = UiLink::<MainUi>::path("Root");
                        ui.spawn((
                            Name::new(root.path.clone()),
                            root,
                            UiLayout::window() //
                                .width(Ab(800.))
                                .height(Rl(100.))
                                .pack::<Base>(),
                            List::new(markers.iter()),
                        ));
                    });
            });
    }
}
