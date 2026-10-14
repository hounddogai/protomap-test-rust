//! `propagate_admissions_status` reads `acme.admissions.v1.AdmissionQueryService/SuggestAdmissions` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::fanportal::v1 as fanportal;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = admissions::admission_query_service_client::AdmissionQueryServiceClient::new(channel.clone());
    let loaded = source
        .suggest_admissions(admissions::SuggestAdmissionsRequest {
            context: Some(context.clone()),
            order_by: "nightly-propagate-admissions-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = AdmissionsStatusRecord {
        status: result.status,
        message: result.message,
    };
    deliver_admissions_status("gateway-policy-workflow", record, channel, &context).await
}

/// What `propagate_admissions_status` keeps of each SuggestAdmissions response.
#[derive(Clone, Debug, Default)]
pub struct AdmissionsStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `propagate_admissions_status` hands each record.
trait AdmissionsStatusSink {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery;
}

/// Writes records to `acme.edgegateway.v1.GatewayPolicyWorkflowService`.
struct GatewayPolicyWorkflowSink {
    client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl AdmissionsStatusSink for GatewayPolicyWorkflowSink {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::ResumeGatewayPolicyWorkflowRequest {
                    reason: record.status,
                    gateway_policy: Some(edgegateway::GatewayPolicy {
                        description: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.resume_gateway_policy_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
}

impl AdmissionsStatusSink for FanSessionSink {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_fan_session(fanportal::CreateFanSessionRequest {
                    fan_session: Some(fanportal::FanSession {
                        fan_session_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.edgegateway.v1.ApiKeyIntegrationService`.
struct ApiKeyIntegrationSink {
    client: edgegateway::api_key_integration_service_client::ApiKeyIntegrationServiceClient<Channel>,
}

impl AdmissionsStatusSink for ApiKeyIntegrationSink {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .disconnect_api_key_source(edgegateway::DisconnectApiKeySourceRequest {
                    api_key_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct AdmissionsStatusLogOnly;

impl AdmissionsStatusSink for AdmissionsStatusLogOnly {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_admissions_status(
    kind: &str,
    record: AdmissionsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AdmissionsStatusSink> = match kind {
        "gateway-policy-workflow" => Box::new(GatewayPolicyWorkflowSink {
            client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "fan-session" => Box::new(FanSessionSink {
            client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone()),
        }),
        "api-key-integration" => Box::new(ApiKeyIntegrationSink {
            client: edgegateway::api_key_integration_service_client::ApiKeyIntegrationServiceClient::new(channel),
        }),
        _ => Box::new(AdmissionsStatusLogOnly),
    };
    sink.deliver(record).await
}
