mod commands;
mod state;

use commands::{
    build_survey_plot, build_survey_scene, calculate_minimum_curvature, get_plot_preferences,
    get_project_audit, get_units, inspect_document, ping, run_scan, save_project, select_project,
};
use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let state = AppState::open(app_data_dir)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            select_project,
            inspect_document,
            save_project,
            get_project_audit,
            get_units,
            get_plot_preferences,
            calculate_minimum_curvature,
            build_survey_plot,
            build_survey_scene,
            run_scan
        ])
        .run(tauri::generate_context!())
        .expect("error while running WellForge");
}
