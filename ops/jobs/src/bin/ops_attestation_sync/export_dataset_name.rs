//! `export_dataset_name` reads `acme.datasets.v1.DatasetService/GetDataset` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::datasets::v1 as datasets;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::ledger::v1 as ledger;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = datasets::dataset_service_client::DatasetServiceClient::new(channel.clone());
    let fetched = source
        .get_dataset(datasets::GetDatasetRequest {
            context: Some(context.clone()),
            dataset_id: "live-export-dataset-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let dataset = fetched.dataset.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = DatasetNameRecord {
        name: acme_kit::text::normalize(&dataset.name),
        dataset_id: acme_kit::text::normalize(&dataset.dataset_id),
        correlation_id: result.correlation_id,
        status: result.status,
    };
    dispatch_dataset_name("vehicle-query", record, channel, &context).await
}

/// What `export_dataset_name` keeps of each GetDataset response.
#[derive(Clone, Debug, Default)]
pub struct DatasetNameRecord {
    pub name: String,
    pub dataset_id: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `export_dataset_name` hands each record.
trait DatasetNameDestination {
    fn forward(&self, record: DatasetNameRecord) -> Delivery;
}

/// Writes records to `acme.ledger.v1.LedgerAccountValidationService`.
struct LedgerAccountValidationSink {
    client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient<Channel>,
}

impl DatasetNameDestination for LedgerAccountValidationSink {
    fn forward(&self, record: DatasetNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_ledger_account_decision(ledger::ExplainLedgerAccountDecisionRequest {
                    ledger_account_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fleet.v1.VehicleQueryService`.
struct VehicleQuerySink {
    client: fleet::vehicle_query_service_client::VehicleQueryServiceClient<Channel>,
}

impl DatasetNameDestination for VehicleQuerySink {
    fn forward(&self, record: DatasetNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_vehicles(fleet::CountVehiclesRequest {
                    filter: Some(fleet::count_vehicles_request::Filter {
                        owner_id: record.dataset_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct DatasetNameLogOnly;

impl DatasetNameDestination for DatasetNameLogOnly {
    fn forward(&self, record: DatasetNameRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.name);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_dataset_name(
    kind: &str,
    record: DatasetNameRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let ledger_account_validation = LedgerAccountValidationSink {
        client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient::new(
            channel.clone(),
        ),
    };
    let vehicle_query = VehicleQuerySink {
        client: fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel),
    };
    let write: &dyn Fn(DatasetNameRecord) -> Delivery = match kind {
        "ledger-account-validation" => &|record| ledger_account_validation.forward(record),
        "vehicle-query" => &|record| vehicle_query.forward(record),
        _ => &|record| DatasetNameLogOnly.forward(record),
    };
    write(record).await
}
