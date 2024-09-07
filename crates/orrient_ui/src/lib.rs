mod console;
pub mod compass;
mod debug_panel;
mod marker_list;

use bevy::prelude::*;

use orrient_core::prelude::*;
use orrient_pathing::prelude::*;

use compass::UiCompassWindowExt as _;
use debug_panel::UiDebugPanelExt as _;
use marker_list::window::UiMarkerWindowExt as _;
use sickle_ui::ui_builder::UiBuilderExt as _;
use sickle_ui::ui_builder::UiRoot;
use sickle_ui::widgets::prelude::*;
use sickle_ui::SickleUiPlugin;

#[derive(Component)]
struct OrrientMenuItem(pub MarkerEvent);

#[derive(Event, Clone, Debug)]
pub enum UiEvent {
    ToggleUI,
    CloseUi,
    CompassSize(UVec2),
    PlayerPosition(Vec2),
    MapPosition(Vec2),
    MapScale(f32),
    MapOpen(bool),
}

fn setup(mut commands: Commands) {
    let camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                ..default()
            },
            ..default()
        })
        .id();

    commands.ui_builder(UiRoot).container(
        (
            NodeBundle {
                background_color: Color::NONE.into(),
                ..default()
            },
            TargetCamera(camera),
        ),
        |container| {
            container.compass();
            container.marker_window();
            container.debug_panel();
        },
    );
}

fn ui_state_system(mut ui_events: EventReader<UiEvent>, mut state: ResMut<NextState<GameState>>) {
    for event in ui_events.read() {
        if let UiEvent::MapOpen(map_open) = event {
            if *map_open {
                state.set(GameState::WorldMap);
            } else {
                state.set(GameState::InGame);
            }
        }
    }
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvent>();

        app.add_plugins(console::Plugin);
        app.add_plugins(SickleUiPlugin);
        app.add_plugins(compass::Plugin);
        app.add_plugins(marker_list::Plugin);
        app.add_plugins(debug_panel::Plugin);
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            ui_state_system.run_if(in_state(AppState::Running).and_then(on_event::<UiEvent>())),
        );
    }
}
