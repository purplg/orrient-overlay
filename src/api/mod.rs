use std::marker::PhantomData;

use bevy::prelude::*;

use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use gw2lib::{
    model::{items::Item, maps::Map, EndpointWithId},
    Client, EndpointError, Requester,
};

#[derive(Clone, Copy, Debug)]
pub enum Request {
    Map(u32),
    Item(u32),
}

#[derive(Clone, Debug)]
pub enum Response {
    Map(Map),
    Item(Item),
}

#[derive(Event, Clone, Debug)]
pub struct RequestComplete(pub Response);

#[derive(Resource, Default)]
pub struct RequestQueue(Vec<Request>);

impl RequestQueue {
    pub fn fetch(&mut self, request: Request) {
        self.0.push(request);
    }
}

pub async fn fetch_map_data(map_id: u32) -> Result<Response, EndpointError> {
    let client = Client::default();
    client.single(map_id).await.map(Response::Map)
}

async fn fetch_item_data(item_id: u32) -> Result<Response, EndpointError> {
    let client = Client::default();
    client.single(item_id).await.map(Response::Item)
}

fn request_task_system(
    mut task_executor: AsyncTaskRunner<Result<Response, EndpointError>>,
    mut queue: ResMut<RequestQueue>,
    mut events: EventWriter<RequestComplete>,
) {
    match task_executor.poll() {
        AsyncTaskStatus::Idle => {
            if let Some(request) = queue.0.pop() {
                info!("Fetching {request:?}");
                match request {
                    Request::Map(map_id) => {
                        task_executor.start(fetch_map_data(map_id));
                    }
                    Request::Item(item_id) => {
                        task_executor.start(fetch_item_data(item_id));
                    }
                }
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(result) => match result {
            Ok(response) => {
                events.send(RequestComplete(response));
            }
            Err(err) => {
                warn!("{err:?}");
            }
        },
    }
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RequestComplete>();
        app.add_systems(Update, request_task_system);
        app.init_resource::<RequestQueue>();
    }
}
