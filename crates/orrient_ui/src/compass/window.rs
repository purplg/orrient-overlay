use bevy::color::palettes;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use sickle_ui::prelude::*;

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_size);
    }
}

#[derive(Component)]
pub struct CompassWindow {
    pub fullscreen: bool,
    pub docked_offset: Vec2,
    pub ui_position: Vec2,
    pub ui_size: Vec2,
    pub map_center: Vec2,
    pub map_scale: f32,
}

impl Default for CompassWindow {
    fn default() -> Self {
        Self {
            fullscreen: false,
            docked_offset: Default::default(),
            ui_position: Default::default(),
            ui_size: Default::default(),
            map_center: Default::default(),
            map_scale: 1.0,
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

    pub fn clamp(&self, map_position: Vec2) -> Vec2 {
        let map_offset = map_position - self.map_center;
        let mut ui_position = map_offset / self.map_scale + self.ui_position + self.ui_size * 0.5;

        if !self.fullscreen {
            let ui_bounds = self.ui_bounds();
            if ui_position.x < ui_bounds.min.x {
                // LEFT
                ui_position.x = ui_bounds.min.x;
            } else if ui_position.x > ui_bounds.max.x - 16.0 {
                // RIGHT
                ui_position.x = ui_bounds.max.x - 16.0;
            }

            if ui_position.y > ui_bounds.max.y - 16.0 {
                // BOTTOM
                ui_position.y = ui_bounds.max.y - 16.0;
            } else if ui_position.y < ui_bounds.min.y {
                // TOP
                ui_position.y = ui_bounds.min.y;
            }
        }

        ui_position
    }

    fn ui_bounds(&self) -> Rect {
        Rect {
            min: Vec2 {
                x: self.ui_right(),
                y: self.ui_top(),
            },
            max: Vec2 {
                x: self.ui_left(),
                y: self.ui_bottom(),
            },
        }
    }

    fn ui_left(&self) -> f32 {
        self.ui_position.x + self.ui_size.x
    }

    fn ui_right(&self) -> f32 {
        self.ui_position.x
    }

    fn ui_top(&self) -> f32 {
        self.ui_position.y
    }

    fn ui_bottom(&self) -> f32 {
        self.ui_position.y + self.ui_size.y
    }
}

pub(super) trait UiCompassWindowExt {
    fn compass(&mut self);
}

impl UiCompassWindowExt for UiBuilder<'_, Entity> {
    fn compass(&mut self) {
        self.container(CompassWindow::frame(), |_| {})
            .insert(CompassWindow {
                docked_offset: Vec2 { x: 0.0, y: 36.0 },
                ..default()
            });
    }
}

fn update_size(
    mut commands: Commands,
    mut q_compass: Query<(Entity, &mut CompassWindow)>,
    mut ui_events: EventReader<UiEvent>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for event in ui_events.read() {
        match event {
            UiEvent::MapOpen(is_open) => {
                let window = window.single();
                let (entity, mut compass) = q_compass.single_mut();
                compass.fullscreen = *is_open;
                if compass.fullscreen {
                    commands
                        .ui_builder(entity)
                        .style()
                        .absolute_position(Vec2::ZERO)
                        .width(Val::Px(window.width()))
                        .height(Val::Px(window.height()));
                } else {
                    commands
                        .ui_builder(entity)
                        .style()
                        .absolute_position(compass.ui_position)
                        .width(Val::Px(compass.ui_size.x))
                        .height(Val::Px(compass.ui_size.y));
                }
            }
            UiEvent::MapPosition(pos) => {
                let (_entity, mut compass) = q_compass.single_mut();
                compass.map_center = *pos;
            }
            UiEvent::CompassSize(size) => {
                let window = window.single();
                let (entity, mut compass) = q_compass.single_mut();
                if compass.fullscreen {
                    compass.ui_position = Vec2 { x: 0.0, y: 0.0 };
                    compass.ui_size = Vec2 {
                        x: window.width(),
                        y: window.height(),
                    };
                } else {
                    compass.ui_position = Vec2 {
                        x: window.width() - size.x as f32 - compass.docked_offset.x,
                        y: window.height() - size.y as f32 - compass.docked_offset.y,
                    };
                    compass.ui_size = Vec2 {
                        x: size.x as f32,
                        y: size.y as f32,
                    };
                }
                commands
                    .ui_builder(entity)
                    .style()
                    .absolute_position(compass.ui_position)
                    .width(Val::Px(compass.ui_size.x))
                    .height(Val::Px(compass.ui_size.y));
            }
            UiEvent::MapScale(scale) => {
                let (_entity, mut compass) = q_compass.single_mut();
                compass.map_scale = *scale;
            }
            _ => {}
        }
    }
}
