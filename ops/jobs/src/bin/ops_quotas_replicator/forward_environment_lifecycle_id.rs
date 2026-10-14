//! `forward_environment_lifecycle_id` reads
//! `acme.configuration.v1.EnvironmentLifecycleService/GetEnvironmentLifecycle` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::datasets::v1 as datasets;
use acme_proto::acme::tours::v1 as tours;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient::new(channel.clone());
    let fetched = source
        .get_environment_lifecycle(configuration::GetEnvironmentLifecycleRequest {
            context: Some(context.clone()),
            environment_id: "live-forward-environment-lifecycle-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let environment = fetched.environment.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = EnvironmentLifecycleIdRecord {
        environment_id: environment.environment_id,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn EnvironmentLifecycleIdTarget>)> = vec![
        (
            "tour",
            Box::new(TourSink {
                client: tours::tour_service_client::TourServiceClient::new(channel.clone()),
            }),
        ),
        (
            "dataset",
            Box::new(DatasetSink {
                client: datasets::dataset_service_client::DatasetServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "tour" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `forward_environment_lifecycle_id` keeps of each GetEnvironmentLifecycle response.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentLifecycleIdRecord {
    pub environment_id: String,
    pub status: String,
}

/// Where `forward_environment_lifecycle_id` hands each record.
trait EnvironmentLifecycleIdTarget {
    fn push(&self, record: EnvironmentLifecycleIdRecord) -> Delivery;
}

/// Hands records to `acme.tours.v1.TourService`.
struct TourSink {
    client: tours::tour_service_client::TourServiceClient<Channel>,
}

impl EnvironmentLifecycleIdTarget for TourSink {
    fn push(&self, record: EnvironmentLifecycleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_tour(tours::CreateTourRequest {
                    tour: Some(tours::Tour {
                        tour_id: record.status,
                        title: record.environment_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.datasets.v1.DatasetService`.
struct DatasetSink {
    client: datasets::dataset_service_client::DatasetServiceClient<Channel>,
}

impl EnvironmentLifecycleIdTarget for DatasetSink {
    fn push(&self, record: EnvironmentLifecycleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_dataset(datasets::DeleteDatasetRequest {
                    etag: record.environment_id,
                    dataset_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
