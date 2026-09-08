use crate::api::Result;
use theseus::telemetry::FrontendErrorReport;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("telemetry")
        .invoke_handler(tauri::generate_handler![
            submit_frontend_error,
            notify_online,
        ])
        .build()
}

#[tauri::command]
pub async fn submit_frontend_error(report: FrontendErrorReport) -> Result<()> {
    theseus::telemetry::submit_frontend_error(report).await?;
    Ok(())
}

#[tauri::command]
pub fn notify_online() {
    theseus::telemetry::notify_online();
}
