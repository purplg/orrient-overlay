use bevy::color::palettes;
use bevy::prelude::*;

use orrient_core::prelude::*;
use orrient_pathing::marker::poi::PoiMarker;
use orrient_pathing::marker::trail::create_trail_mesh;
use orrient_pathing::marker::trail::TrailMaterial;
use orrient_pathing::marker::trail::TrailMesh;
use orrient_pathing::prelude::*;

use bevy_console::clap::Parser;
use bevy_console::AddConsoleCommand;
use bevy_console::ConsoleCommand;
use bevy_console::ConsolePlugin;
use clap::Subcommand;
use clap::ValueEnum;

use crate::compass::marker::CompassMarker;
use crate::UiEvent;

/// List all the markers available
#[derive(Parser, ConsoleCommand)]
#[command(name = "unloadall")]
struct UnloadAllCommand;
fn unload_all_command(
    mut log: ConsoleCommand<UnloadAllCommand>,
    mut events: EventWriter<MarkerEvent>,
) {
    if let Some(Ok(UnloadAllCommand)) = log.take() {
        events.send(MarkerEvent::DisableAll);
        log.reply_ok("Unloaded all markers");
    }
}

/// Override the current map_id
#[derive(Parser, ConsoleCommand)]
#[command(name = "mapid")]
struct MapIdCommand {
    new_id: u32,
}
fn mapid_command(mut log: ConsoleCommand<MapIdCommand>, mut commands: Commands) {
    if let Some(Ok(MapIdCommand { new_id })) = log.take() {
        commands.insert_resource(MapId(new_id));
        log.reply_ok(format!("Loaded map_id {new_id}"));
    }
}

/// List the loaded marker packs
#[derive(Parser, ConsoleCommand)]
#[command(name = "packs")]
struct PacksCommand;
fn packs_command(mut log: ConsoleCommand<PacksCommand>, packs: Res<MarkerPacks>) {
    if let Some(Ok(PacksCommand)) = log.take() {
        for id in packs.keys() {
            log.reply(&id.0);
        }
        log.reply_ok("");
    }
}

#[derive(ValueEnum, Clone, Copy)]
enum System {
    Compass,
}
/// Initialize some systems to some default value
#[derive(Parser, ConsoleCommand)]
#[command(name = "setup")]
struct SetupCommand {
    system: System,
}
fn setup_command(mut log: ConsoleCommand<SetupCommand>, mut ui_events: EventWriter<UiEvent>) {
    if let Some(Ok(SetupCommand { system })) = log.take() {
        match system {
            System::Compass => {
                ui_events.send(UiEvent::CompassSize(UVec2 { x: 362, y: 362 }));
            }
        }
        log.reply_ok("");
    }
}

/// Add things.
#[derive(Parser, ConsoleCommand)]
#[command(name = "add")]
struct AddCommand {
    #[command(subcommand)]
    kind: Add,
}

#[derive(Component)]
struct CreatedByConsole;

#[derive(Subcommand, Clone)]
enum Add {
    Marker { x: f32, y: f32 },
}
fn add_command(
    mut log: ConsoleCommand<AddCommand>,
    mut commands: Commands,
    packs: Res<MarkerPacks>,
) {
    if let Some(Ok(AddCommand { kind })) = log.take() {
        match kind {
            Add::Marker { x, y } => {
                let icon = packs.values().next().unwrap().get_images().next().unwrap();

                commands.spawn((
                    CreatedByConsole,
                    Transform::from_translation(Vec3::new(x, 0.0, y)), //
                    icon.clone(),
                    PoiMarker,
                ));
            }
        }
        log.reply_ok("");
    }
}

/// Delete things.
#[derive(Parser, ConsoleCommand)]
#[command(name = "delete")]
struct DeleteCommand {
    #[command(subcommand)]
    kind: Delete,
}
#[derive(Subcommand, Clone)]
enum Delete {
    Markers,
}
fn delete_command(
    mut log: ConsoleCommand<DeleteCommand>,
    mut commands: Commands,
    mut q_items: Query<Entity, With<CreatedByConsole>>,
    q_markers: Query<(Entity, &CompassMarker)>,
) {
    if let Some(Ok(DeleteCommand { kind })) = log.take() {
        match kind {
            Delete::Markers => {
                for item in &q_items
                    .transmute_lens_filtered::<Entity, With<PoiMarker>>()
                    .query()
                {
                    commands.entity(item).despawn_recursive();
                    if let Some((entity, _)) = q_markers.iter().find(|(_, marker)| marker.0 == item)
                    {
                        commands.entity(entity).despawn_recursive();
                    } else {
                        log.reply("Could not find associated compass marker.");
                    }
                }
            }
        }
        log.ok();
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "marker")]
struct MarkerCommand {
    #[command(subcommand)]
    subcommand: MarkerSubcommand,
}
#[derive(Subcommand, Clone)]
enum MarkerSubcommand {
    Load { pack_id: String, marker_id: String },
}
fn marker_command(mut log: ConsoleCommand<MarkerCommand>, mut events: EventWriter<MarkerEvent>) {
    if let Some(Ok(MarkerCommand { subcommand: kind })) = log.take() {
        match kind {
            MarkerSubcommand::Load { pack_id, marker_id } => {
                let full_id = FullMarkerId {
                    pack_id: PackId(pack_id),
                    marker_id: MarkerId(marker_id.into()),
                };
                events.send(MarkerEvent::Enabled(full_id));
                log.ok();
            }
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "trail")]
struct TrailCommand {
    #[command(subcommand)]
    kind: Trail,
}
#[derive(Subcommand, Clone)]
enum Trail {
    Load { pack_id: String, marker_id: String },
}
fn trail_command(
    mut log: ConsoleCommand<TrailCommand>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut trail_materials: ResMut<Assets<TrailMaterial>>,
    packs: Res<MarkerPacks>,
    map_id: Res<MapId>,
) {
    if let Some(Ok(TrailCommand { kind })) = log.take() {
        match kind {
            Trail::Load { pack_id, marker_id } => {
                let full_id = FullMarkerId {
                    pack_id: PackId(pack_id),
                    marker_id: MarkerId(marker_id.into()),
                };
                let Some(pack) = packs.get(&full_id.pack_id) else {
                    log.reply_failed("Pack not found");
                    return;
                };

                let Some(trails) = pack.get_trails(&full_id.marker_id) else {
                    log.reply_failed("Trail not found for marker_id: {full_id}");
                    return;
                };

                debug!("Loading trails for {}...", full_id);

                for trail in trails.iter().filter(|trail| trail.map_id == map_id.0) {
                    let iter = trail.path.iter().rev().map(|path| Vec3 {
                        x: path.x,
                        y: path.y,
                        z: -path.z,
                    });

                    let Some(texture) = pack.get_image(&trail.texture_file) else {
                        warn!("Could not find texture {}", trail.texture_file);
                        continue;
                    };

                    debug!("Trail texture: {:?}", trail.texture_file);

                    let material = trail_materials.add(TrailMaterial {
                        color: palettes::basic::WHITE.into(),
                        color_texture: Some(texture),
                        alpha_mode: AlphaMode::Blend,
                        speed: 1.0,
                    });

                    let mesh = create_trail_mesh(iter);

                    commands.spawn((
                        TrailMesh,
                        MaterialMeshBundle {
                            mesh: meshes.add(mesh),
                            material,
                            ..default()
                        },
                    ));
                }
                info!("Trail {} loaded.", full_id);
            }
        }
        log.ok();
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin);
        app.add_console_command::<UnloadAllCommand, _>(unload_all_command);
        app.add_console_command::<MapIdCommand, _>(mapid_command);
        app.add_console_command::<PacksCommand, _>(packs_command);
        app.add_console_command::<SetupCommand, _>(setup_command);
        app.add_console_command::<AddCommand, _>(add_command);
        app.add_console_command::<DeleteCommand, _>(delete_command);
        app.add_console_command::<MarkerCommand, _>(marker_command);
        app.add_console_command::<TrailCommand, _>(trail_command);
    }
}
