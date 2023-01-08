use bevy::prelude::{App, Res, ResMut, State, SystemSet, Resource};

use crate::AppState;

use self::model_loader::add_model_loading;

pub mod model_loader;

const NUM_LOADERS: u32 = 1;

#[derive(Resource)]
struct LoadingInfo {
    loaded: u32,
}

pub fn add_loading_methods(app: &mut App) -> &mut App {
    app
        .insert_resource(LoadingInfo { loaded: 0 })
        .add_system_set(SystemSet::on_update(AppState::Setup)
            .with_system(check_done_loading));
    
    add_model_loading(app);
    app
}

fn check_done_loading(info: Res<LoadingInfo>, mut state: ResMut<State<AppState>>) {
    if info.loaded >= NUM_LOADERS {
        state.set(AppState::Running).unwrap();
    }
}
