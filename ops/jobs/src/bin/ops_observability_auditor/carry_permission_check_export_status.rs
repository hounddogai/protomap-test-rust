//! `carry_permission_check_export_status` reads
//! `acme.authorization.v1.PermissionCheckExportService/SharePermissionCheckExport` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::permission_check_export_service_client::PermissionCheckExportServiceClient::new(channel.clone());
    let fetched = source
        .share_permission_check_export(authorization::SharePermissionCheckExportRequest {
            context: Some(context.clone()),
            permission_check_id: "nightly-carry-permission-check-export-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let permission_check = fetched.permission_check.unwrap_or_default();
    let record = PermissionCheckExportStatusRecord {
        status: result.status,
        reference_id: outcome.reference_id,
        correlation_id: acme_kit::text::normalize(&permission_check.correlation_id),
    };
    let sinks: Vec<(&str, Box<dyn PermissionCheckExportStatusDestination>)> = vec![
        (
            "key-ring-reporting",
            Box::new(KeyRingReportingSink {
                client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "gateway-policy-query",
            Box::new(GatewayPolicyQuerySink {
                client: edgegateway::gateway_policy_query_service_client::GatewayPolicyQueryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "flag-evaluation-admin",
            Box::new(FlagEvaluationAdminSink {
                client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "key-ring-reporting" {
            continue;
        }
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `carry_permission_check_export_status` keeps of each SharePermissionCheckExport response.
#[derive(Clone, Debug, Default)]
pub struct PermissionCheckExportStatusRecord {
    pub status: String,
    pub reference_id: String,
    pub correlation_id: String,
}

/// Where `carry_permission_check_export_status` hands each record.
trait PermissionCheckExportStatusDestination {
    fn forward(&self, record: PermissionCheckExportStatusRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRingReportingService`.
struct KeyRingReportingSink {
    client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient<Channel>,
    context: common::RequestContext,
}

impl PermissionCheckExportStatusDestination for KeyRingReportingSink {
    fn forward(&self, record: PermissionCheckExportStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ShareKeyRingReportRequest {
                    reason: record.reference_id,
                    details: Some(keymanagement::share_key_ring_report_request::Details {
                        template_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.share_key_ring_report(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.GatewayPolicyQueryService`.
struct GatewayPolicyQuerySink {
    client: edgegateway::gateway_policy_query_service_client::GatewayPolicyQueryServiceClient<Channel>,
}

impl PermissionCheckExportStatusDestination for GatewayPolicyQuerySink {
    fn forward(&self, record: PermissionCheckExportStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_gateway_policies(edgegateway::SummarizeGatewayPoliciesRequest {
                    details: Some(edgegateway::summarize_gateway_policies_request::Details {
                        cron: record.reference_id,
                        ..Default::default()
                    }),
                    gateway_policy_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationAdminService`.
struct FlagEvaluationAdminSink {
    client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient<Channel>,
}

impl PermissionCheckExportStatusDestination for FlagEvaluationAdminSink {
    fn forward(&self, record: PermissionCheckExportStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .archive_flag_evaluation(featureflags::ArchiveFlagEvaluationRequest {
                    reason: record.correlation_id,
                    details: Some(featureflags::archive_flag_evaluation_request::Details {
                        reason_code: record.reference_id,
                        ..Default::default()
                    }),
                    flag_evaluation_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
