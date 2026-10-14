//! `carry_role_bindings_token` reads `acme.authorization.v1.RoleBindingQueryService/SuggestRoleBindings` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::role_binding_query_service_client::RoleBindingQueryServiceClient::new(channel.clone());
    let reply = source
        .suggest_role_bindings(authorization::SuggestRoleBindingsRequest {
            context: Some(context.clone()),
            order_by: "live-carry-role-bindings-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = reply.page_info.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = RoleBindingsTokenRecord {
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        message: result.message,
    };
    dispatch_role_bindings_token("key-rotation-workflow", record, channel, &context).await
}

/// What `carry_role_bindings_token` keeps of each SuggestRoleBindings response.
#[derive(Clone, Debug, Default)]
pub struct RoleBindingsTokenRecord {
    pub next_page_token: String,
    pub message: String,
}

/// Where `carry_role_bindings_token` hands each record.
trait RoleBindingsTokenDestination {
    fn push(&self, record: RoleBindingsTokenRecord) -> Delivery;
}

/// Passes records to `acme.audit.v1.AuditEventService`.
struct AuditEventSink {
    client: audit::audit_event_service_client::AuditEventServiceClient<Channel>,
}

impl RoleBindingsTokenDestination for AuditEventSink {
    fn push(&self, record: RoleBindingsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_audit_event(audit::CloneAuditEventRequest {
                    audit_event: Some(audit::AuditEvent {
                        audit_event_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingsTokenDestination for SponsorQuerySink {
    fn push(&self, record: RoleBindingsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::LookupSponsorRequest {
                    sponsor_id: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_sponsor(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl RoleBindingsTokenDestination for KeyRotationWorkflowSink {
    fn push(&self, record: RoleBindingsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_key_rotation_workflow(keymanagement::AdvanceKeyRotationWorkflowRequest {
                    key_rotation: Some(keymanagement::KeyRotation {
                        title: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketReportingService`.
struct QuotaBucketReportingSink {
    client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingsTokenDestination for QuotaBucketReportingSink {
    fn push(&self, record: RoleBindingsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ScheduleQuotaBucketReportRequest {
                    quota_bucket_id: record.message,
                    reason: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.schedule_quota_bucket_report(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct RoleBindingsTokenLogOnly;

impl RoleBindingsTokenDestination for RoleBindingsTokenLogOnly {
    fn push(&self, record: RoleBindingsTokenRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.next_page_token);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_role_bindings_token(
    kind: &str,
    record: RoleBindingsTokenRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let audit_event = AuditEventSink {
        client: audit::audit_event_service_client::AuditEventServiceClient::new(channel.clone()),
    };
    let sponsor_query = SponsorQuerySink {
        client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let key_rotation_workflow = KeyRotationWorkflowSink {
        client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(
            channel.clone(),
        ),
    };
    let quota_bucket_reporting = QuotaBucketReportingSink {
        client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(RoleBindingsTokenRecord) -> Delivery = match kind {
        "audit-event" => &|record| audit_event.push(record),
        "sponsor-query" => &|record| sponsor_query.push(record),
        "key-rotation-workflow" => &|record| key_rotation_workflow.push(record),
        "quota-bucket-reporting" => &|record| quota_bucket_reporting.push(record),
        _ => &|record| RoleBindingsTokenLogOnly.push(record),
    };
    write(record).await
}
