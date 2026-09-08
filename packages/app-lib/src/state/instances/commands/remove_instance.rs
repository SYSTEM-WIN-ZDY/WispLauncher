use crate::state::State;
use crate::state::instances::adapters::sqlite::instance_rows;
use crate::state::instances::config_sync;
use crate::util::io;

pub(crate) async fn remove_instance(
    instance_id: &str,
    state: &State,
) -> crate::Result<()> {
    let _instance_lock = state.lock_instance_content(instance_id).await;

    let instance = instance_rows::get_instance_by_id(instance_id, &state.pool)
        .await?
        .ok_or_else(|| {
            crate::ErrorKind::InputError("Unknown instance".to_string())
        })?;

    let path = state.directories.instances_dir().join(&instance.path);
    if path.exists() {
        io::remove_dir_all(&path).await?;
    }

    let jobs = crate::install::store::mark_instance_deleted(instance_id, state)
        .await?;
    instance_rows::delete_instance_by_id(&instance.id, &state.pool).await?;
    config_sync::remove_config_file(&state.directories, &instance.path).await?;
    for job in jobs {
        if let Err(error) =
            crate::install::events::emit_install_job(&job.snapshot()).await
        {
            tracing::warn!(
                "Failed to emit deleted instance download state: {error}"
            );
        }
    }

    Ok(())
}
