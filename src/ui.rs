use bevy::{ecs::system::EntityCommands, prelude::*, window::PrimaryWindow};

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, icon_button_system);
        app.add_systems(
            Update,
            toggle_hittest_system.run_if(on_event::<OrrientEvent>()),
        );
    }
}

const ICON_WIDTH: f32 = 28.;
const ICON_HEIGHT: f32 = 28.;
const ICON_POSITION: Vec2 = Vec2 {
    x: 323.,
    // Icons are centered on the 16th pixel from the top
    y: 16. - ICON_HEIGHT * 0.5,
};

fn setup(mut commands: Commands) {
    MainCanvas::insert(&mut commands.spawn_empty());
}

trait UIElement: Component + Default {
    fn insert(entity: &mut EntityCommands) {
        Self::build(entity);
        entity.insert(Self::default());
    }

    fn build(entity: &mut EntityCommands);

    fn as_child(entity: &mut ChildBuilder) {
        Self::build(&mut entity.spawn(Self::default()));
    }
}

#[derive(Component, Default)]
struct MainCanvas;

impl UIElement for MainCanvas {
    fn build(entity: &mut EntityCommands) {
        entity
            .insert(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                visibility: Visibility::Visible,
                ..default()
            })
            .with_children(MainIcon::as_child);
    }
}

#[derive(Component, Default)]
struct MainIcon;

impl UIElement for MainIcon {
    fn build(entity: &mut EntityCommands) {
        entity
            .insert(NodeBundle {
                style: Style {
                    left: Val::Px(ICON_POSITION.x),
                    top: Val::Px(ICON_POSITION.y),
                    width: Val::Px(ICON_WIDTH),
                    height: Val::Px(ICON_HEIGHT),
                    border: UiRect::all(Val::Px(2.)),
                    ..default()
                },
                background_color: Color::srgb(0.65, 0.65, 0.65).into(),
                ..default()
            })
            .with_children(MainIconButton::as_child);
    }
}

#[derive(Component, Default)]
struct MainIconButton;

impl UIElement for MainIconButton {
    fn build(entity: &mut EntityCommands) {
        entity.insert(ButtonBundle {
            button: Button,
            style: Style {
                width: Val::Px(ICON_WIDTH),
                height: Val::Px(ICON_HEIGHT),
                ..default()
            },
            ..default()
        });
    }
}

fn icon_button_system(button: Query<&Interaction, (Changed<Interaction>, With<MainIconButton>)>) {
    let Ok(button) = button.get_single() else {
        return;
    };

    match *button {
        Interaction::Pressed => println!("click"),
        Interaction::Hovered => {}
        Interaction::None => {}
    }
}

fn toggle_hittest_system(
    mut events: EventReader<OrrientEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui: Query<&mut Visibility, With<MainCanvas>>,
) {
    for event in events.read() {
        if let OrrientEvent::ToggleUI = event {
            let mut window = window.single_mut();
            window.cursor.hit_test = !window.cursor.hit_test;

            window.decorations = window.cursor.hit_test;
            *ui.single_mut() = if window.cursor.hit_test {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            println!(
                "UI {}",
                if window.cursor.hit_test {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
    }
}
