mod theme;
use theme::*;

use bevy::prelude::*;
use sickle_ui::prelude::*;

struct RepoPack {
    name: String,
    description: String,
    url: String,
    browse: String,
    filename: String,
    categories: String,
    version: String,
    size: f32,
    downloads: usize,
    author_name: String,
    author_username: String,
    last_update: String,
}

/// The main view for the entire downloads tab area.
#[derive(Component)]
pub(super) struct DownloadsView;

trait UiEntryExt {
    fn entry(&mut self, repo_pack: RepoPack) -> UiBuilder<Entity>;
}
impl UiEntryExt for UiBuilder<'_, Entity> {
    fn entry(&mut self, repo_pack: RepoPack) -> UiBuilder<Entity> {
        self.container(DownloadPackMain::frame(), |parent| {
            parent.container(Content::frame(), |parent| {
                parent.container(Header::frame(), |parent| {
                    parent.spawn((
                        Title,
                        TextBundle::from_section(
                            repo_pack.name,
                            TextStyle {
                                font_size: 20.,
                                ..default()
                            },
                        ),
                    ));
                });

                parent.container(Body::frame(), |parent| {
                    parent.spawn(TextBundle::from_section(
                        repo_pack.description,
                        TextStyle {
                            font_size: 14.,
                            ..default()
                        },
                    ));
                });

                parent.container(Footer::frame(), |parent| {
                    parent.container(Categories::frame(), |parent| {
                        parent.spawn(TextBundle::from_section(
                            repo_pack.categories,
                            TextStyle {
                                font_size: 14.,
                                ..default()
                            },
                        ));
                    });

                    parent.container(Buttons::frame(), |parent| {
                        parent.repo_button("Info");
                        parent.repo_button("Download");
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

fn setup(trigger: Trigger<OnAdd, DownloadsView>, mut commands: Commands) {
    let mut test_data: Vec<RepoPack> = Default::default();
    test_data.push(RepoPack {
        name: "[fast] TacO Markers".into(),
        description: "A set of markers and trails for some of the most profitable solo-farming gathering routes across a wide range of maps.\r\nClick the Info button for current benchmarks and more details!".into(),
        url: "https://mp-repo.blishhud.com/packs/fast_TacO_pack.taco".into(),
        browse: "https://fast.farming-community.eu/farming/guides/fast-taco-marker".into(),
        filename: "fast_TacO_pack.taco".into(),
        categories: "Solo Farming".into(),
        version: "https://fast.farming-community.eu/fast/markers/fast_TacO_pack.taco".into(),
        size: 0.63182163,
        downloads: 468,
        author_name: "[fast]".into(),
        author_username: "".into(),
        last_update: "2024-08-27T13:15:17.1130223Z".into(),
    });
    test_data.push(RepoPack {
        name: "[FvD] Dungeon Markers".into(),
        description: "Trails for all dungeon explorable and story paths (excluding Arah Story mode). Includes: Path info, mechanics, and some skips.".into(),
        url: "https://mp-repo.blishhud.com/packs/FvD_Dungeon_Guide.taco".into(),
        browse: "https://github.com/SZG5/gw2-dungeon-markers".into(),
        filename: "FvD_Dungeon_Guide.taco".into(),
        categories: "Dungeons".into(),
        version: "178128182".into(),
        size: 1.26542,
        downloads: 34,
        author_name: "Z. Long".into(),
        author_username: "S Z G.4359".into(),
        last_update: "2024-07-07T03:01:23".into(),
    });

    let mut builder = commands.ui_builder(trigger.entity());
    for pack in test_data {
        builder.row(|parent| {
            parent.entry(pack);
        });
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(theme::Plugin);
        app.observe(setup);
    }
}
