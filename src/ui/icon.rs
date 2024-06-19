use bevy::prelude::*;

use bevy::ecs::system::EntityCommands;

use super::UIElement;

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, icon_button_system);
    }
}

const ICON_WIDTH: f32 = 26.;
const ICON_HEIGHT: f32 = 26.;
const ICON_POSITION: Vec2 = Vec2 {
    x: 323.,
    // Icons are centered on the 16th pixel from the top
    y: 16. - ICON_HEIGHT * 0.5,
};

#[derive(Component, Default)]
pub(super) struct MainIcon;

impl UIElement for MainIcon {
    fn build(&self, entity: &mut EntityCommands) {
        entity.insert(ButtonBundle {
            button: Button,
            style: Style {
                left: Val::Px(ICON_POSITION.x),
                top: Val::Px(ICON_POSITION.y),
                width: Val::Px(ICON_WIDTH),
                height: Val::Px(ICON_HEIGHT),
                ..default()
            },
            ..default()
        });
    }
}

fn icon_button_system(button: Query<&Interaction, (Changed<Interaction>, With<MainIcon>)>) {
    let Ok(button) = button.get_single() else {
        return;
    };

    match *button {
        Interaction::Pressed => println!("click"),
        Interaction::Hovered => {}
        Interaction::None => {}
    }
}
