#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

#[macro_use]
extern crate log;

use serde::{Deserialize, Serialize};

pub use uncode_config::UncodeConfig;
use crate::workspace_config::WorkspaceConfig;

mod cmd;

mod uncode_config;
mod workspace_config;

#[derive(Serialize)]
struct Reply {
  data: String,
}

#[derive(Serialize, Deserialize)]
struct SummaryConfig {
  pub workspace: WorkspaceConfig,
  pub uncode: UncodeConfig,
}

#[derive(Serialize, Deserialize)]
struct EventPayload {
  event_type: String,
  data: String,
}

fn main() {
  setup_log();

  let uncode = UncodeConfig::read_config();
  let workspace = if !uncode.workspace_config.is_empty() {
    WorkspaceConfig::from_path(uncode.workspace_config.clone())
  } else {
    WorkspaceConfig::default()
  };

  let uncode_config = SummaryConfig {
    uncode,
    workspace,
  };

  tauri::Builder::default()
    .on_page_load(move |window, _| {
      window.listen("js_event".to_string(), move |event| {
        info!("event_id {:}, event_payload: {:?}", event.id(), event.payload());
        let payload: EventPayload = serde_json::from_str(event.payload().expect("lost payload")).expect("uncode no match model");
        match payload.event_type.as_str() {
          "save_config" => {
            let config: SummaryConfig = serde_json::from_str(&payload.data).expect("unable to convert config");
            UncodeConfig::save_config(config.uncode);
          }
          &_ => {}
        }
      });

      window
        .emit(&"bootstrap".to_string(), Some(serde_json::to_string(&uncode_config).unwrap()))
        .expect("failed to emit");
    })
    .invoke_handler(tauri::generate_handler![
      cmd::log_operation,
      cmd::perform_request,
      cmd::save_workspace,
      cmd::set_title,

      // story
      cmd::get_story,


      cmd::open_file,
      cmd::open_dir,

      // design
      cmd::get_design,
      cmd::build_modeling,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn setup_log() {
  use tracing_subscriber::prelude::*;
  let filter_layer = tracing_subscriber::filter::LevelFilter::DEBUG;
  let fmt_layer = tracing_subscriber::fmt::layer()
    .with_target(true);

  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();
}
