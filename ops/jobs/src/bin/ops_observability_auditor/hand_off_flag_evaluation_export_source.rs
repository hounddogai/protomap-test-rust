//! `hand_off_flag_evaluation_export_source` reads
//! `acme.featureflags.v1.FlagEvaluationExportService/GetFlagEvaluationExport` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient::new(channel.clone());
    let found = source
        .get_flag_evaluation_export(featureflags::GetFlagEvaluationExportRequest {
            context: Some(context.clone()),
            flag_evaluation_id: "nightly-hand-off-flag-evaluation-export-source".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_evaluation = found.flag_evaluation.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = OffFlagEvaluationExportSourceRecord {
        source: acme_kit::text::normalize(&flag_evaluation.source),
        correlation_id: result.correlation_id,
        kind: flag_evaluation.kind,
        message: result.message,
    };
    deliver_off_flag_evaluation_export_source("access-grant-validation", record, channel, &context).await
}

/// What `hand_off_flag_evaluation_export_source` keeps of each GetFlagEvaluationExport response.
#[derive(Clone, Debug, Default)]
pub struct OffFlagEvaluationExportSourceRecord {
    pub source: String,
    pub correlation_id: String,
    pub kind: String,
    pub message: String,
}

/// Where `hand_off_flag_evaluation_export_source` hands each record.
trait OffFlagEvaluationExportSourceDestination {
    fn send(&self, record: OffFlagEvaluationExportSourceRecord) -> Delivery;
}

/// Writes records to `acme.edgegateway.v1.RouteBindingQueryService`.
struct RouteBindingQuerySink {
    client: edgegateway::route_binding_query_service_client::RouteBindingQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffFlagEvaluationExportSourceDestination for RouteBindingQuerySink {
    fn send(&self, record: OffFlagEvaluationExportSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::BrowseRouteBindingsRequest {
                    filter: Some(edgegateway::browse_route_bindings_request::Filter {
                        owner_id: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_route_bindings(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
}

impl OffFlagEvaluationExportSourceDestination for AccessGrantValidationSink {
    fn send(&self, record: OffFlagEvaluationExportSourceRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_access_grant(secrets::ScoreAccessGrantRequest {
                    access_grant_id: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OffFlagEvaluationExportSourceLogOnly;

impl OffFlagEvaluationExportSourceDestination for OffFlagEvaluationExportSourceLogOnly {
    fn send(&self, record: OffFlagEvaluationExportSourceRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.source);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_off_flag_evaluation_export_source(
    kind: &str,
    record: OffFlagEvaluationExportSourceRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn OffFlagEvaluationExportSourceDestination> = match kind {
        "route-binding-query" => Box::new(RouteBindingQuerySink {
            client: edgegateway::route_binding_query_service_client::RouteBindingQueryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "access-grant-validation" => Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(channel),
        }),
        _ => Box::new(OffFlagEvaluationExportSourceLogOnly),
    };
    sink.send(record).await
}
