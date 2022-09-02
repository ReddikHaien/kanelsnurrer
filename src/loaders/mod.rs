use bevy::prelude::{App, Res, ResMut, State, SystemSet};

use crate::AppState;

use self::model_loader::load_models;

pub mod model_loader;

const NUM_LOADERS: u32 = 1;

struct LoadingInfo {
    loaded: u32,
}

pub fn add_loading_methods(app: &mut App) -> &mut App {
    app.insert_resource(LoadingInfo { loaded: 0 })
        .add_system_set(SystemSet::on_enter(AppState::Setup).with_system(load_models))
        .add_system_set(SystemSet::on_update(AppState::Setup).with_system(check_done_loading));

    app
}

fn check_done_loading(info: Res<LoadingInfo>, mut state: ResMut<State<AppState>>) {
    if info.loaded >= NUM_LOADERS {
        state.set(AppState::Running).unwrap();
    }
}
