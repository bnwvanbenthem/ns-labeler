use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Error, ResourceExt};
use serde_json::{json, Value};
use tracing::*;

use crate::crd::{Labeler, LabelerStatus};

pub async fn patch(
    client: Client,
    name: &str,
    namespace: &str,
    success: bool,
) -> Result<Labeler, Error> {
    let api: Api<Labeler> = Api::namespaced(client, namespace);

    let data: Value = json!({
        "status": LabelerStatus { succeeded: success }
    });

    api.patch_status(name, &PatchParams::default(), &Patch::Merge(&data))
        .await
}

pub async fn print(client: Client, name: &str, namespace: &str) -> Result<(), Error> {
    let api: Api<Labeler> = Api::namespaced(client, namespace);

    let cdb = api.get_status(name).await?;

    info!(
        "Got status succeeded {:?} for custom resource {} in namespace {}",
        cdb.clone()
            .status
            .unwrap_or(LabelerStatus { succeeded: false })
            .succeeded,
        cdb.name_any(),
        namespace
    );

    Ok(())
}