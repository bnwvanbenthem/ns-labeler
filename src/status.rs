use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Error, ResourceExt};
use serde_json::{json, Value};
use tracing::*;

use crate::crd::{Tagger, TaggerStatus};

pub async fn patch(
    client: Client,
    name: &str,
    namespace: &str,
    success: bool,
    tagged: Vec<String>,
) -> Result<Tagger, Error> {
    let api: Api<Tagger> = Api::namespaced(client, namespace);

    let data: Value = json!({
        "status": TaggerStatus { succeeded: success, tagged: tagged },
    });

    api.patch_status(name, &PatchParams::default(), &Patch::Merge(&data))
        .await
}

pub async fn print(client: Client, name: &str, namespace: &str) -> Result<(), Error> {
    let api: Api<Tagger> = Api::namespaced(client, namespace);

    let cdb = api.get_status(name).await?;

    info!(
        "Got status succeeded {:?} for custom resource {} in namespace {}",
        cdb.clone()
            .status
            .unwrap_or(TaggerStatus {
                succeeded: false,
                tagged: Vec::default()
            })
            .succeeded,
        cdb.name_any(),
        namespace
    );

    Ok(())
}
