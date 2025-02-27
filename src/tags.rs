use crate::crd::Tagger;

use kube::{client::Client, Api};
use serde_json::json;
use std::{collections::BTreeMap, sync::Arc};
use tracing::*;

use k8s_openapi::api::core::v1::Namespace;
use kube::api::{Patch, PatchParams};

const TAGGER: &str = "tagger.cncp.nl";

pub async fn apply_tags(
    cr: Arc<Tagger>,
    client: Client,
    namespace: Namespace,
) -> Result<(), kube::Error> {
    let tagger = cr.as_ref();
    let ns = namespace.clone();
    let ns_name = namespace.metadata.name.unwrap_or("unknown".to_string());

    let mut needs_update = false;
    let mut updated_ns = ns.clone();

    // Handle labels
    {
        let mut labels = updated_ns.metadata.labels.unwrap_or_default();
        for label in &tagger.spec.labels {
            let current_value = labels.get(&label.key);
            if current_value != Some(&label.value) {
                needs_update = true;
                labels.insert(label.key.clone(), label.value.clone());
            }
        }
        updated_ns.metadata.labels = Some(labels);
    }

    // Handle annotations
    {
        let mut annotations = updated_ns.metadata.annotations.unwrap_or_default();
        for annotation in &tagger.spec.annotations {
            let current_value = annotations.get(&annotation.key);
            if current_value != Some(&annotation.value) {
                needs_update = true;
                annotations.insert(annotation.key.clone(), annotation.value.clone());
            }
        }
        updated_ns.metadata.annotations = Some(annotations);
    }

    let namespaces: Api<Namespace> = Api::all(client.clone());

    if needs_update {
        let patch = Patch::Merge(&updated_ns);
        namespaces
            .patch(&ns_name, &PatchParams::default(), &patch)
            .await?;
        info!("Updated tags on namespace: {}", ns_name);
    }

    Ok(())
}

pub async fn delete_tags(
    cr: Arc<Tagger>,
    client: Client,
    namespace: Namespace,
) -> Result<(), kube::Error> {
    let tagger = cr.as_ref();
    let ns = namespace.clone();
    let ns_name = namespace.metadata.name.unwrap_or("unknown".to_string());

    // Check if the namespace is in the exclude list
    if tagger.spec.excludelist.contains(&ns_name) {
        let mut needs_update = false;

        // Prepare patch objects
        let mut labels_patch = serde_json::Map::new();
        let mut annotations_patch = serde_json::Map::new();

        // Handle labels removal
        {
            let mut labels = ns.metadata.labels.unwrap_or_default();
            for label in &tagger.spec.labels {
                let current_value = labels.get(&label.key);
                if current_value == Some(&label.value) {
                    needs_update = true;
                    labels.remove(&label.key);
                    labels_patch.insert(label.key.clone(), json!(null)); // Explicitly set to null
                }
            }
        }

        // Handle annotations removal
        {
            let mut annotations = ns.metadata.annotations.unwrap_or_default();
            for annotation in &tagger.spec.annotations {
                let current_value = annotations.get(&annotation.key);
                if current_value == Some(&annotation.value) {
                    needs_update = true;
                    annotations.remove(&annotation.key);
                    annotations_patch.insert(annotation.key.clone(), json!(null));
                    // Explicitly set to null
                }
            }
        }

        let namespaces: Api<Namespace> = Api::all(client.clone());

        if needs_update {
            // Construct a patch with explicit null removals
            let patch = json!({
                "metadata": {
                    "labels": labels_patch,
                    "annotations": annotations_patch
                }
            });
            namespaces
                .patch(&ns_name, &PatchParams::default(), &Patch::Merge(&patch))
                .await?;
            info!("Removed tags from namespace: {}", ns_name);
        }
    }
    // If not in excludelist, do nothing
    Ok(())
}

pub async fn apply_tagged_true_annotation(
    client: Client,
    namespace: Namespace,
) -> Result<(), kube::Error> {
    let ns_name = namespace.metadata.name.unwrap_or("unknown".to_string());
    let annotations = namespace.metadata.annotations.unwrap_or_default();

    if !check_for_tagged_annotation(annotations) {
        // Define the patch with the annotation
        let patch = json!({
            "metadata": {
                "annotations": {
                    TAGGER.to_string(): "tagged"
                }
            }
        });

        let namespaces: Api<Namespace> = Api::all(client.clone());

        // Apply the patch directly to the specified namespace
        namespaces
            .patch(&ns_name, &PatchParams::default(), &Patch::Merge(&patch))
            .await?;

        info!(
            "Applied Status annotation tagger.cncp.nl=tagged to namespace {}",
            &ns_name
        );
    }

    Ok(())
}

pub async fn delete_tagged_true_annotation(
    client: Client,
    namespace: Namespace,
) -> Result<(), kube::Error> {
    let ns_name = namespace.metadata.name.unwrap_or("unknown".to_string());
    let annotations = namespace.metadata.annotations.unwrap_or_default();

    if check_for_tagged_annotation(annotations) {
        // Define the patch with the annotation
        let patch = json!({
            "metadata": {
                "annotations": {
                    TAGGER.to_string(): null
                }
            }
        });

        let namespaces: Api<Namespace> = Api::all(client.clone());

        // Apply the patch directly to the specified namespace
        namespaces
            .patch(&ns_name, &PatchParams::default(), &Patch::Merge(&patch))
            .await?;

        info!(
            "Removed Status annotation tagger.cncp.nl=tagged to namespace {}",
            &ns_name
        );
    }

    Ok(())
}

fn check_for_tagged_annotation(annotations: BTreeMap<String, String>) -> bool {
    // check if tagger annotion exists
    let mut tagger_annotation = false;

    for annotation in annotations {
        if annotation.0 == TAGGER.to_string() {
            tagger_annotation = true;
        }
    }

    tagger_annotation
}

pub async fn get_all_tagged_true_annotations(client: Client) -> Result<Vec<String>, kube::Error> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let namespace_list = namespaces.list(&Default::default()).await?;

    let mut tagged_list: Vec<String> = Vec::new();

    for namespace in namespace_list.items {
        let ns_name = namespace.metadata.name.unwrap_or("unknown".to_string());
        let annotations = namespace.metadata.annotations.unwrap_or_default();

        for annotation in annotations {
            if annotation.0 == TAGGER.to_string() {
                tagged_list.push(ns_name.clone());
            }
        }
    }
    Ok(tagged_list)
}
