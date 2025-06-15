use rocket::{self, routes, fs::FileServer};
use rocket_contrib::templates::Template;
use std::sync::{Arc, Mutex};

use crate::config::SimulationConfig;
use crate::model::simulation::ProductionModel;
use crate::stats::collector::SimulationStats;
use crate::web::routes::{index, run_simulation, update_config, get_config, view_results};

pub struct AppState {
    pub config: Mutex<SimulationConfig>,
    pub latest_results: Mutex<Option<SimulationStats>>,
}

pub async fn start_server(initial_config: SimulationConfig) -> Result<(), rocket::Error> {
    let app_state = Arc::new(AppState {
        config: Mutex::new(initial_config),
        latest_results: Mutex::new(None),
    });

    rocket::build()
        .mount("/", routes![index, run_simulation, update_config, get_config, view_results])
        .mount("/static", FileServer::from("static"))
        .manage(app_state)
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
}