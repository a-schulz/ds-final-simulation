use rocket::{get, post, State, response::Redirect};
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket_contrib::templates::Template;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::config::SimulationConfig;
use crate::model::simulation::ProductionModel;
use crate::stats::collector::SimulationStats;
use crate::web::server::AppState;

#[get("/")]
pub fn index(state: &State<Arc<AppState>>) -> Template {
    let mut context = HashMap::new();

    // Aktuelle Konfiguration für die Anzeige
    let config = state.config.lock().unwrap();
    context.insert("config", &*config);

    // Prüfen, ob Ergebnisse vorhanden sind
    let has_results = state.latest_results.lock().unwrap().is_some();
    context.insert("has_results", &has_results);

    Template::render("index", &context)
}

#[derive(FromForm, Debug)]
pub struct ConfigForm {
    smd_time_minutes: f64,
    lotbad_batch_size: usize,
    lotbad_batches_per_hour: usize,
    montage_min_minutes: f64,
    montage_max_minutes: f64,
    qc_mean_minutes: f64,
    qc_std_dev_minutes: f64,
    qc_min_minutes: f64,
    buffer_capacity: usize,
    montage_stations: usize,
    test_stations: usize,
    simulation_duration_hours: f64,
    warm_up_period_hours: f64,
    replications: usize,
}

#[post("/config", data = "<form>")]
pub fn update_config(form: Form<ConfigForm>, state: &State<Arc<AppState>>) -> Redirect {
    let new_config = SimulationConfig {
        smd_time_minutes: form.smd_time_minutes,
        lotbad_batch_size: form.lotbad_batch_size,
        lotbad_batches_per_hour: form.lotbad_batches_per_hour,
        montage_min_minutes: form.montage_min_minutes,
        montage_max_minutes: form.montage_max_minutes,
        qc_mean_minutes: form.qc_mean_minutes,
        qc_std_dev_minutes: form.qc_std_dev_minutes,
        qc_min_minutes: form.qc_min_minutes,
        buffer_capacity: form.buffer_capacity,
        montage_stations: form.montage_stations,
        test_stations: form.test_stations,
        simulation_duration_hours: form.simulation_duration_hours,
        warm_up_period_hours: form.warm_up_period_hours,
        replications: form.replications,
    };

    let mut config = state.config.lock().unwrap();
    *config = new_config;

    Redirect::to(uri!(index))
}

#[get("/config")]
pub fn get_config(state: &State<Arc<AppState>>) -> Json<SimulationConfig> {
    let config = state.config.lock().unwrap().clone();
    Json(config)
}

#[post("/run")]
pub async fn run_simulation(state: &State<Arc<AppState>>) -> Redirect {
    // Konfiguration abrufen
    let config = state.config.lock().unwrap().clone();

    // Simulation in einem separaten Thread ausführen
    let stats = tokio::task::spawn_blocking(move || {
        let mut stats_collector = SimulationStats::new();

        for i in 0..config.replications {
            let mut model = ProductionModel::new(config.clone());
            model.run_simulation();
            stats_collector.add_run_results(model);
        }

        stats_collector
    }).await.unwrap();

    // Ergebnisse speichern
    let mut results = state.latest_results.lock().unwrap();
    *results = Some(stats);

    Redirect::to(uri!(view_results))
}

#[get("/results")]
pub fn view_results(state: &State<Arc<AppState>>) -> Template {
    let mut context = HashMap::new();

    if let Some(stats) = &*state.latest_results.lock().unwrap() {
        context.insert("stats", stats);
        context.insert("has_results", &true);
    } else {
        context.insert("has_results", &false);
    }

    Template::render("results", &context)
}