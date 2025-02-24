use nslabeler::crd::Labeler;
use nslabeler::status;

use futures::stream::StreamExt;
use kube::runtime::watcher::Config;
use kube::{client::Client, runtime::controller::Action, runtime::Controller, Api};
use kube::ResourceExt;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::*;

/// Context injected with each `reconcile` and `on_error` method invocation.
struct ContextData {
    /// Kubernetes client to make Kubernetes API requests with. Required for K8S resource management.
    client: Client,
}

impl ContextData {
    /// Constructs a new instance of ContextData.
    ///
    /// # Arguments:
    /// - `client`: A Kubernetes client to make Kubernetes REST API requests with. Resources
    /// will be created and deleted with this client.
    pub fn new(client: Client) -> Self {
        ContextData { client }
    }
}

#[tokio::main]
async fn main() -> Result<(), kube::Error> {
    tracing_subscriber::fmt::init();
    // First, a Kubernetes client must be obtained using the `kube` crate
    // The client will later be moved to the custom controller
    let kubeconfig: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    // Preparation of resources used by the `kube_runtime::Controller`
    let crd_api: Api<Labeler> = Api::all(kubeconfig.clone());
    let context: Arc<ContextData> = Arc::new(ContextData::new(kubeconfig.clone()));

    // The controller comes from the `kube_runtime` crate and manages the reconciliation process.
    // It requires the following information:
    // - `kube::Api<T>` this controller "owns". In this case, `T = Labeler`, as this controller owns the `Labeler` resource,
    // - `kube::runtime::watcher::Config` can be adjusted for precise filtering of `Labeler` resources before the actual reconciliation, e.g. by label,
    // - `reconcile` function with reconciliation logic to be called each time a resource of `Labeler` kind is created/updated/deleted,
    // - `on_error` function to call whenever reconciliation fails.
    Controller::new(crd_api.clone(), Config::default())
        .run(reconcile, on_error, context)
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(custom_resource) => {
                    info!("Reconciliation successful. Resource: {:?}", custom_resource);
                }
                Err(reconciliation_err) => {
                    error!("Reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;

    Ok(())
}

async fn reconcile(cr: Arc<Labeler>, context: Arc<ContextData>) -> Result<Action, Error> {
    let client: Client = context.client.clone(); // The `Client` is shared -> a clone from the reference is obtained

    // The resource of `Labeler` kind is required to have a namespace set. However, it is not guaranteed
    // the resource will have a `namespace` set. Therefore, the `namespace` field on object's metadata
    // is optional and Rust forces the programmer to check for it's existence first.
    let namespace: String = match cr.namespace() {
        None => {
            // If there is no namespace to deploy to defined, reconciliation ends with an error immediately.
            return Err(Error::UserInputError(
                "Expected Labeler resource to be namespaced. Can't deploy to an unknown namespace."
                    .to_owned(),
            ));
        }
        // If namespace is known, proceed. In a more advanced version of the operator, perhaps
        // the namespace could be checked for existence first.
        Some(namespace) => namespace,
    };

    let name = cr.name_any(); // Name of the Labeler resource is used to name the subresources as well.


    // Label logic !!!!!!!!!!!!!!!!!!!!!!!!!!
    {
        info!("\n ! Labeling all the namespaces !\n");

        // Patch the status to true
        match status::patch(client.clone(), &name, &namespace, true).await {
            Ok(labeler) => {
                info!("Successfully updated Labeler status '{}': {:?}", name, labeler.status);
            }
            Err(e) => {
                error!("Failed to update Labeler status '{}': {:?}", name, e);
                return Err(e.into());
            }
        }

        status::print(client.clone(), &name, &namespace).await?;

        Ok(Action::requeue(Duration::from_secs(30)))

    }
}

fn on_error(cr: Arc<Labeler>, error: &Error, context: Arc<ContextData>) -> Action {
    // Clone the necessary data
    let client = context.client.clone();

    let name = String::from(&cr.name_any());
    let namespace = String::from(
        &cr.metadata
            .namespace
            .clone()
            .unwrap_or(String::from("default")),
    );
    // Use the existing Tokio runtime to spawn the async task
    tokio::spawn(async move {
        match status::patch(client, &name, &namespace, false).await {
            Ok(_) => {
                info!("Updated status with reconcile error")
            }
            Err(e) => {
                // Update status failed, handle the error
                error!("Failed to update status: {:?}", e);
            }
        }
    });

    // Continue with the rest of your on_error logic
    error!("Reconciliation error:\n{:?}.\n{:?}", error, cr);
    Action::requeue(Duration::from_secs(5))
}

/// All errors possible to occur during reconciliation
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },
    /// Error in user input or Labeler resource definition, typically missing fields.
    #[error("Invalid Labeler CRD: {0}")]
    UserInputError(String),
}
pub type Result<T, E = Error> = std::result::Result<T, E>;
