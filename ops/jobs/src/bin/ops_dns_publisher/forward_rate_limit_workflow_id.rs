//! `forward_rate_limit_workflow_id` reads
//! `acme.ratelimiting.v1.RateLimitWorkflowService/RetryRateLimitWorkflow` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::procurement::v1 as procurement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(channel.clone());
    let loaded = source
        .retry_rate_limit_workflow(ratelimiting::RetryRateLimitWorkflowRequest {
            context: Some(context.clone()),
            rate_limit_id: "nightly-forward-rate-limit-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let rate_limit = loaded.rate_limit.unwrap_or_default();
    let record = RateLimitWorkflowIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        description: acme_kit::text::normalize(&rate_limit.description),
    };
    deliver_rate_limit_workflow_id("certificate-authority-validation", record, channel, &context).await
}

/// What `forward_rate_limit_workflow_id` keeps of each RetryRateLimitWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct RateLimitWorkflowIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub description: String,
}

/// Where `forward_rate_limit_workflow_id` hands each record.
trait RateLimitWorkflowIdSink {
    fn send(&self, record: RateLimitWorkflowIdRecord) -> Delivery;
}

/// Writes records to `acme.audit.v1.AuditEventService`.
struct AuditEventSink {
    client: audit::audit_event_service_client::AuditEventServiceClient<Channel>,
}

impl RateLimitWorkflowIdSink for AuditEventSink {
    fn send(&self, record: RateLimitWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_audit_events(audit::ListAuditEventsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    filter: Some(audit::list_audit_events_request::Filter {
                        owner_id: record.message,
                        region: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CertificateAuthorityValidationService`.
struct CertificateAuthorityValidationSink {
    client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient<
        Channel,
    >,
}

impl RateLimitWorkflowIdSink for CertificateAuthorityValidationSink {
    fn send(&self, record: RateLimitWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_certificate_authority_decision(certificates::ExplainCertificateAuthorityDecisionRequest {
                    certificate_authority: Some(certificates::CertificateAuthority {
                        parent_id: record.message,
                        ..Default::default()
                    }),
                    details: Some(certificates::explain_certificate_authority_decision_request::Details {
                        locale: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.procurement.v1.PurchaseRequisitionQueryService`.
struct PurchaseRequisitionQuerySink {
    client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient<Channel>,
}

impl RateLimitWorkflowIdSink for PurchaseRequisitionQuerySink {
    fn send(&self, record: RateLimitWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_purchase_requisitions(procurement::CountPurchaseRequisitionsRequest {
                    order_by: record.description,
                    filter: Some(procurement::count_purchase_requisitions_request::Filter {
                        query: record.message,
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.subscriptions.v1.SubscriptionService`.
struct SubscriptionSink {
    client: subscriptions::subscription_service_client::SubscriptionServiceClient<Channel>,
}

impl RateLimitWorkflowIdSink for SubscriptionSink {
    fn send(&self, record: RateLimitWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_subscription(subscriptions::CreateSubscriptionRequest {
                    subscription: Some(subscriptions::Subscription {
                        subscription_id: record.correlation_id,
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
struct RateLimitWorkflowIdLogOnly;

impl RateLimitWorkflowIdSink for RateLimitWorkflowIdLogOnly {
    fn send(&self, record: RateLimitWorkflowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_rate_limit_workflow_id(
    kind: &str,
    record: RateLimitWorkflowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn RateLimitWorkflowIdSink> = match kind {
        "audit-event" => Box::new(AuditEventSink { client: audit::audit_event_service_client::AuditEventServiceClient::new(channel.clone()) }),
        "certificate-authority-validation" => Box::new(CertificateAuthorityValidationSink { client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient::new(channel.clone()) }),
        "purchase-requisition-query" => Box::new(PurchaseRequisitionQuerySink { client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(channel.clone()) }),
        "subscription" => Box::new(SubscriptionSink { client: subscriptions::subscription_service_client::SubscriptionServiceClient::new(channel) }),
        _ => Box::new(RateLimitWorkflowIdLogOnly),
    };
    sink.send(record).await
}
