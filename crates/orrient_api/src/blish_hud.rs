use std::time::Duration;

use bevy::prelude::*;

use bevy::utils::HashMap;
use bevy_async_task::AsyncTaskRunner;
use bevy_async_task::AsyncTaskStatus;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

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
enum UpdateState {
    #[default]
    Idle,
    Queued,
    Updating,
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
    mut state: ResMut<NextState<UpdateState>>,
) {
    for event in er_bhupdate.read() {
        match event {
            BHAPIEvent::Refresh => {
                state.set(UpdateState::Queued);
            }
            BHAPIEvent::Download(repo_pack_id) => todo!(),
        }
    }
}

#[derive(Error, Debug)]
pub enum APIError {
    #[error("Remote returned error")]
    Reqwest(#[from] reqwest::Error),

    #[error("Could not deserialize data")]
    Json(#[from] serde_json::Error),
}

pub async fn fetch_repo_data() -> Result<Vec<RepoPack>, APIError> {
    let client = Client::default();
    let response = client.get(BH_URL).send().await.map_err(APIError::Reqwest)?;
    let body = response.text().await.map_err(APIError::Reqwest)?;
    let repo_pack: Vec<RepoPack> = serde_json::from_str(&body).map_err(APIError::Json)?;
    Ok(repo_pack)
}

pub async fn fetch_test_repo_data() -> Result<Vec<RepoPack>, APIError> {
    let repo_pack: Vec<RepoPack> = serde_json::from_str(&TEST_DATA).map_err(APIError::Json)?;
    Ok(repo_pack)
}

fn task_system(
    mut commands: Commands,
    mut task_executor: AsyncTaskRunner<Result<Vec<RepoPack>, APIError>>,
    state: Res<State<UpdateState>>,
    mut next_state: ResMut<NextState<UpdateState>>,
) {
    match task_executor.poll() {
        AsyncTaskStatus::Idle => {
            if UpdateState::Queued == *state.get() {
                info!("Updating repos...");
                task_executor.start(fetch_repo_data());
                next_state.set(UpdateState::Updating);
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(result) => {
            match result {
                Ok(packs) => {
                    let mut available_packs = AvailablePacks::default();
                    for (i, pack) in packs.iter().enumerate() {
                        available_packs.insert(RepoPackId(i), pack.clone());
                    }
                    debug!("Updated repos.");
                    commands.insert_resource(available_packs);
                }
                Err(err) => {
                    warn!("{err:?}");
                }
            }
            next_state.set(UpdateState::Idle);
        }
    }
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BHAPIEvent>();
        app.init_state::<UpdateState>();
        app.init_resource::<AvailablePacks>();
        app.add_systems(Update, event_system.run_if(on_event::<BHAPIEvent>()));
        app.add_systems(Update, task_system);
    }
}
