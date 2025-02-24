use crate::crd::Labeler;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Error};
use serde_json::{json, Value};


pub async fn add(client: Client, name: &str, namespace: &str) -> Result<Labeler, Error> {
    let api: Api<Labeler> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": ["labelers.cndev.nl/finalizer"]
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}


pub async fn delete(client: Client, name: &str, namespace: &str) -> Result<Labeler, Error> {
    let api: Api<Labeler> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}
