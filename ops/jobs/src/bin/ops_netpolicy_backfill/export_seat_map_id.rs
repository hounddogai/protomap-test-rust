//! `export_seat_map_id` reads `acme.seating.v1.SeatMapCalculationService/EstimateSeatMap` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient::new(channel.clone());
    let fetched = source
        .estimate_seat_map(seating::EstimateSeatMapRequest {
            context: Some(context.clone()),
            seat_map_id: "current-export-seat-map-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let seat_map = fetched.seat_map.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = SeatMapIdRecord {
        parent_id: acme_kit::text::normalize(&seat_map.parent_id),
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn SeatMapIdDestination>> = BTreeMap::new();
    sinks.insert(
        "crypto-key-approval",
        Box::new(CryptoKeyApprovalSink {
            client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "work-order-calculation",
        Box::new(WorkOrderCalculationSink {
            client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "experiment-calculation",
        Box::new(ExperimentCalculationSink {
            client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "document-validation",
        Box::new(DocumentValidationSink {
            client: documents::document_validation_service_client::DocumentValidationServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("work-order-calculation") else {
        anyhow::bail!("no sink registered for work-order-calculation");
    };
    sink.apply(record).await
}

/// What `export_seat_map_id` keeps of each EstimateSeatMap response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapIdRecord {
    pub parent_id: String,
    pub message: String,
}

/// Where `export_seat_map_id` hands each record.
trait SeatMapIdDestination {
    fn apply(&self, record: SeatMapIdRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyApprovalService`.
struct CryptoKeyApprovalSink {
    client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapIdDestination for CryptoKeyApprovalSink {
    fn apply(&self, record: SeatMapIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::RejectCryptoKeyRequest {
                    reason: record.parent_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.reject_crypto_key(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.maintenance.v1.WorkOrderCalculationService`.
struct WorkOrderCalculationSink {
    client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SeatMapIdDestination for WorkOrderCalculationSink {
    fn apply(&self, record: SeatMapIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(maintenance::CompareWorkOrderOptionsRequest {
                    details: Some(maintenance::compare_work_order_options_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_work_order_options(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
}

impl SeatMapIdDestination for ExperimentCalculationSink {
    fn apply(&self, record: SeatMapIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_experiment(experiments::EstimateExperimentRequest {
                    experiment: Some(experiments::Experiment {
                        experiment_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.documents.v1.DocumentValidationService`.
struct DocumentValidationSink {
    client: documents::document_validation_service_client::DocumentValidationServiceClient<Channel>,
}

impl SeatMapIdDestination for DocumentValidationSink {
    fn apply(&self, record: SeatMapIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_document(documents::PreviewDocumentRequest {
                    document: Some(documents::Document {
                        document_id: record.parent_id,
                        ..Default::default()
                    }),
                    details: Some(documents::preview_document_request::Details {
                        locale: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
