use std::fs::File;
use std::io::Write as _;

use bevy::prelude::*;

use bevy::utils::HashMap;

use bevy_mod_reqwest::BevyReqwest;
use bevy_mod_reqwest::ReqwestErrorEvent;
use bevy_mod_reqwest::ReqwestResponseEvent;
use orrient_pathing::prelude::ReloadMarkersEvent;
use serde::Deserialize;

const BH_URL: &'static str = "https://mp-repo.blishhud.com/repo.json";

const TEST_DATA: &'static str = r#"[
  {
    "Name": "[fast] TacO Markers",
    "Description": "A set of markers and trails for some of the most profitable solo-farming gathering routes across a wide range of maps.\r\nClick the Info button for current benchmarks and more details!",
    "Download": "https://mp-repo.blishhud.com/packs/fast_TacO_pack.taco",
    "Info": "https://fast.farming-community.eu/farming/guides/fast-taco-marker",
    "FileName": "fast_TacO_pack.taco",
    "Categories": "Solo Farming",
    "Version": "https://fast.farming-community.eu/fast/markers/fast_TacO_pack.taco",
    "Size": 0.63182163,
    "TotalDownloads": 468,
    "AuthorName": "[fast]",
    "AuthorUsername": "",
    "LastUpdate": "2024-08-27T13:15:17.1130223Z",
    "DistinctDownloads": {}
  },
  {
    "Name": "[FvD] Dungeon Markers",
    "Description": "Trails for all dungeon explorable and story paths (excluding Arah Story mode). Includes: Path info, mechanics, and some skips.",
    "Download": "https://mp-repo.blishhud.com/packs/FvD_Dungeon_Guide.taco",
    "Info": "https://github.com/SZG5/gw2-dungeon-markers",
    "FileName": "FvD_Dungeon_Guide.taco",
    "Categories": "Dungeons",
    "Version": "178128182",
    "Size": 1.26542,
    "TotalDownloads": 34,
    "AuthorName": "Z. Long",
    "AuthorUsername": "S Z G.4359",
    "LastUpdate": "2024-07-07T03:01:23",
    "DistinctDownloads": {
      "178128182%0.6.0": 2698
    }
  }
]"#;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct AvailablePacks(HashMap<RepoPackId, RepoPack>);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RepoPack {
    pub name: String,
    pub description: String,
    pub download: String,
    pub info: String,
    pub file_name: String,
    pub categories: String,
    pub version: Option<String>,
    pub size: f32,
    pub total_downloads: usize,
    pub author_name: String,
    pub author_username: String,
    pub last_update: String,
}

#[derive(States, Clone, Copy, Hash, PartialEq, Eq, Default, Debug)]
enum RefreshState {
    #[default]
    Idle,
    Queued,
    WaitingForResponse,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RepoPackId(pub usize);

#[derive(Event, Clone, Debug)]
pub enum BHAPIEvent {
    Refresh,
    Download(RepoPackId),
}

fn event_system(
    mut er_bhupdate: EventReader<BHAPIEvent>,
    mut state: ResMut<NextState<RefreshState>>,
) {
    for event in er_bhupdate.read() {
        match event {
            BHAPIEvent::Refresh => {
                state.set(RefreshState::Queued);
            }
            BHAPIEvent::Download(_) => {}
        }
    }
}

fn update_request(mut client: BevyReqwest, mut next_state: ResMut<NextState<RefreshState>>) {
    let request = client.get(BH_URL).build().unwrap();
    client
        .send(request)
        .on_response(
            |trigger: Trigger<ReqwestResponseEvent>,
             mut next_state: ResMut<NextState<RefreshState>>,
             mut available_packs: ResMut<AvailablePacks>| {
                next_state.set(RefreshState::Idle);
                let response = trigger.event();
                let status = response.status();
                if status != 200 {
                    warn!("HTTP error/not ok: {response:?}");
                }

                let Ok(body) = response.as_str() else {
                    warn!("HTTPError/no body: {response:?}");
                    return;
                };

                let Ok(packs) = serde_json::from_str::<Vec<RepoPack>>(&body) else {
                    warn!("HTTPError/deser: {response:?}");
                    return;
                };

                available_packs.clear();
                for (i, pack) in packs.iter().enumerate() {
                    available_packs.insert(RepoPackId(i), pack.clone());
                }
            },
        )
        .on_error(
            |trigger: Trigger<ReqwestErrorEvent>,
             mut next_state: ResMut<NextState<RefreshState>>| {
                next_state.set(RefreshState::Idle);
                let e = &trigger.event().0;
                warn!("Error {e:?}");
            },
        );
    next_state.set(RefreshState::WaitingForResponse);
}

fn download_request(
    mut client: BevyReqwest,
    mut er_api_event: EventReader<BHAPIEvent>,
    available_packs: Res<AvailablePacks>,
) {
    for event in er_api_event.read() {
        let BHAPIEvent::Download(pack_id) = event else {
            continue;
        };

        let Some(repo_pack) = available_packs.get(pack_id).cloned() else {
            warn!("Repo pack not found.");
            continue;
        };

        let request = client.get(repo_pack.download.clone()).build().unwrap();
        client
            .send(request)
            .on_response(
                move |trigger: Trigger<ReqwestResponseEvent>,
                      mut ew_markers: EventWriter<ReloadMarkersEvent>| {
                    let response = trigger.event();
                    let status = response.status();
                    if status != 200 {
                        warn!("Invalid HTTP response: {response:?}");
                    }

                    let Some(base_dirs) = directories::BaseDirs::new() else {
                        error!("No base directory set");
                        return;
                    };

                    let config_dir = base_dirs.config_dir();

                    let dir = config_dir.join("orrient").join("markers");
                    let filepath = dir.join(&repo_pack.file_name);

                    if let Err(err) = std::fs::create_dir_all(dir) {
                        error!("Error when trying to download a marker pack: {err:?}");
                        return;
                    };

                    debug!("Downloaded new pack to: {:?}", filepath);
                    match File::create(filepath) {
                        Ok(mut file) => {
                            if let Err(err) = file.write(response.body()) {
                                error!("Error when writing downloaded marker pack: {err:?}")
                            }
                        }
                        Err(err) => {
                            error!("Error when writing downloaded marker pack: {err:?}")
                        }
                    }

                    ew_markers.send(ReloadMarkersEvent);
                },
            )
            .on_error(|trigger: Trigger<ReqwestErrorEvent>| {
                let e = &trigger.event().0;
                warn!("Error {e:?}");
            });
    }
}

pub(super) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BHAPIEvent>();
        app.init_state::<RefreshState>();
        app.init_resource::<AvailablePacks>();

        app.add_plugins(bevy_mod_reqwest::ReqwestPlugin::default());

        app.add_systems(OnEnter(RefreshState::Queued), update_request);
        app.add_systems(Update, event_system.run_if(on_event::<BHAPIEvent>()));
        app.add_systems(Update, download_request.run_if(on_event::<BHAPIEvent>()));
    }
}