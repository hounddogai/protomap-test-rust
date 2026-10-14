//! `relay_audit_event_eligibility_id` reads
//! `acme.audit.v1.AuditEventValidationService/CheckAuditEventEligibility` and hands what it keeps to the sink
//! of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        audit::audit_event_validation_service_client::AuditEventValidationServiceClient::new(channel.clone());
    let reply = source
        .check_audit_event_eligibility(audit::CheckAuditEventEligibilityRequest {
            context: Some(context.clone()),
            audit_event_id: "main-relay-audit-event-eligibility-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let audit_event = reply.audit_event.unwrap_or_default();
    let record = AuditEventEligibilityIdRecord {
        audit_event_id: audit_event.audit_event_id,
        kind: audit_event.kind,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AuditEventEligibilityIdWriter>> = BTreeMap::new();
    sinks.insert(
        "gateway-policy-workflow",
        Box::new(GatewayPolicyWorkflowSink {
            client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "environment-workflow",
        Box::new(EnvironmentWorkflowSink {
            client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("environment-workflow") else {
        anyhow::bail!("no sink registered for environment-workflow");
    };
    sink.send(record).await
}

/// What `relay_audit_event_eligibility_id` keeps of each CheckAuditEventEligibility response.
#[derive(Clone, Debug, Default)]
pub struct AuditEventEligibilityIdRecord {
    pub audit_event_id: String,
    pub kind: String,
}

/// Where `relay_audit_event_eligibility_id` hands each record.
trait AuditEventEligibilityIdWriter {
    fn send(&self, record: AuditEventEligibilityIdRecord) -> Delivery;
}

/// Sends records to `acme.edgegateway.v1.GatewayPolicyWorkflowService`.
struct GatewayPolicyWorkflowSink {
    client: edgegateway::gateway_policy_workflow_service_client::GatewayPolicyWorkflowServiceClient<Channel>,
}

impl AuditEventEligibilityIdWriter for GatewayPolicyWorkflowSink {
    fn send(&self, record: AuditEventEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resume_gateway_policy_workflow(edgegateway::ResumeGatewayPolicyWorkflowRequest {
                    details: Some(edgegateway::resume_gateway_policy_workflow_request::Details {
                        step: record.audit_event_id,
                        ..Default::default()
                    }),
                    reason: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
}

impl AuditEventEligibilityIdWriter for EnvironmentWorkflowSink {
    fn send(&self, record: AuditEventEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_environment_workflow(configuration::RetryEnvironmentWorkflowRequest {
                    details: Some(configuration::retry_environment_workflow_request::Details {
                        initiated_by: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
