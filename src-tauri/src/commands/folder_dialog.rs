use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;

/// Opens the native folder picker without holding a thread while the user decides.
pub(super) async fn pick_folder(app: &AppHandle) -> Option<String> {
    let (sender, receiver) = oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = sender.send(folder.map(|path| path.to_string()));
    });
    receiver.await.ok().flatten()
}
