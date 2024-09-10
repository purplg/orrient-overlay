mod theme;
use theme::*;

use orrient_api::prelude::*;

use bevy::prelude::*;

use cyborgtime::format_duration;
use itertools::Itertools;
use sickle_ui::prelude::*;

#[derive(Component, Debug)]
enum ButtonKind {
    Info,
    Download,
    Refresh,
}

trait UiEntryExt {
    fn entry(&mut self, pack_id: RepoPackId, repo_pack: &RepoPack) -> UiBuilder<Entity>;
}
impl UiEntryExt for UiBuilder<'_, Entity> {
    fn entry(&mut self, pack_id: RepoPackId, repo_pack: &RepoPack) -> UiBuilder<Entity> {
        self.container(DownloadPackMain::frame(), |parent| {
            parent.container(Content::frame(), |parent| {
                parent.container(Header::frame(), |parent| {
                    parent.spawn((
                        Title,
                        TextBundle::from_section(
                            repo_pack.name.clone(),
                            TextStyle {
                                font_size: 20.,
                                ..default()
                            },
                        ),
                    ));
                    let age: String = repo_pack
                        .last_update
                        .parse::<cyborgtime::Timestamp>()
                        .ok()
                        .and_then(|timestamp| timestamp.elapsed().ok())
                        .map(format_duration)
                        .map(|formatted_time| formatted_time.to_string())
                        .map(|text| text.split(" ").take(2).join(" "))
                        .map(|text| format!("{text} ago"))
                        .unwrap_or_else(|| repo_pack.last_update.clone());
                    parent.spawn((
                        Timestamp,
                        TextBundle::from_section(
                            age,
                            TextStyle {
                                font_size: 16.,
                                ..default()
                            },
                        )
                        .with_text_justify(JustifyText::Right),
                    ));
                });

                parent.container(Body::frame(), |parent| {
                    parent.spawn(TextBundle::from_section(
                        repo_pack.description.clone(),
                        TextStyle {
                            font_size: 14.,
                            ..default()
                        },
                    ));
                });

                parent.container(Footer::frame(), |parent| {
                    parent.container(Categories::frame(), |parent| {
                        parent.spawn(TextBundle::from_section(
                            repo_pack.categories.clone(),
                            TextStyle {
                                font_size: 14.,
                                ..default()
                            },
                        ));
                    });

                    parent.container(Buttons::frame(), |parent| {
                        parent
                            .repo_button("Info")
                            .insert((pack_id, ButtonKind::Info));
                        parent
                            .repo_button("Download")
                            .insert((pack_id, ButtonKind::Download));
                    });
                });
            });
        })
    }
}

trait UiRepoButtonExt {
    fn repo_button(&mut self, label: impl Into<String>) -> UiBuilder<Entity>;
}
impl UiRepoButtonExt for UiBuilder<'_, Entity> {
    fn repo_button(&mut self, label: impl Into<String>) -> UiBuilder<Entity> {
        self.container(RepoButton::frame(), |parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 14.,
                    ..default()
                },
            ));
        })
    }
}

fn repo_button(
    mut commands: Commands,
    q_button: Query<(Entity, Option<&RepoPackId>, &ButtonKind, &Interaction), Changed<Interaction>>,
    downloadable_packs: Res<DownloadablePacks>,
    mut ew_bhupdate: EventWriter<BHAPIEvent>,
) {
    for (entity, pack_id, button_kind, interaction) in &q_button {
        let Interaction::Pressed = interaction else {
            continue;
        };

        match button_kind {
            ButtonKind::Info => {
                let pack_id = pack_id.expect("Info buttons should always have a RepoPackId");
                let Some(pack) = downloadable_packs.get(pack_id) else {
                    continue;
                };

                println!("browse: {:?}", pack.info);
            }
            ButtonKind::Download => {
                let pack_id = pack_id.expect("Download buttons should always have a RepoPackId");
                ew_bhupdate.send(BHAPIEvent::Download(*pack_id));
                commands
                    .entity(entity)
                    .add_pseudo_state(PseudoState::Disabled);
            }
            ButtonKind::Refresh => {
                ew_bhupdate.send(BHAPIEvent::Refresh);
            }
        }
    }
}

fn setup(trigger: Trigger<OnAdd, DownloadsView>, mut commands: Commands) {
    let mut builder = commands.ui_builder(trigger.entity());
    builder.container(RepoBar::frame(), |parent| {
        parent.repo_button("Refresh").insert(ButtonKind::Refresh);
    });
}

fn update_repos(
    mut commands: Commands,
    q_downloads_view: Query<Entity, With<DownloadsView>>,
    downloadable_packs: Res<DownloadablePacks>,
) {
    let Ok(entity) = q_downloads_view.get_single() else {
        return;
    };
    let mut builder = commands.ui_builder(entity);
    builder.scroll_view(Some(ScrollAxis::Vertical), |parent| {
        let sorted_packs = downloadable_packs
            .iter()
            .sorted_by(|(_, pack_a), (_, pack_b)| pack_a.name.cmp(&pack_b.name));
        for (pack_id, pack) in sorted_packs {
            parent.row(|parent| {
                parent.entry(*pack_id, pack);
            });
        }
    });
}

pub(super) use theme::DownloadsView;
pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(theme::Plugin);
        app.add_systems(Update, repo_button);
        app.add_systems(
            Update,
            update_repos.run_if(resource_exists_and_changed::<DownloadablePacks>),
        );
        app.observe(setup);
    }
}
