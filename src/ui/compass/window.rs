use bevy::{color::palettes, prelude::*, window::PrimaryWindow};
use sickle_ui::prelude::*;

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_size);
    }
}

#[derive(Component)]
struct CompassWindow {
    offset: Vec2,
}

impl Default for CompassWindow {
    fn default() -> Self {
        CompassWindow {
            offset: Vec2 { x: 0.0, y: 36.0 }, // TODO support other UI scales besides Normal
        }
    }
}

impl CompassWindow {
    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(1.)),
                ..default()
            },
            border_color: palettes::basic::RED.into(),
            ..default()
        }
    }
}

pub trait UiCompassWindowExt {
    fn compass(&mut self);
}

impl UiCompassWindowExt for UiBuilder<'_, Entity> {
    fn compass(&mut self) {
        self.container(CompassWindow::frame(), |parent| {})
            .insert(CompassWindow::default());
    }
}

fn update_size(
    mut commands: Commands,
    q_compass: Query<(Entity, &CompassWindow)>,
    mut ui_events: EventReader<UiEvent>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for event in ui_events.read() {
        match event {
            UiEvent::CompassSize(size) => {
                let window = window.single();
                let (entity, compass) = q_compass.single();
                commands
                    .ui_builder(entity)
                    .style()
                    .absolute_position(Vec2 {
                        x: window.width() - size.x as f32 - compass.offset.x,
                        y: window.height() - size.y as f32 - compass.offset.y,
                    })
                    .width(Val::Px(size.x as f32))
                    .height(Val::Px(size.y as f32));
            }
            _ => {}
        }
    }
}
