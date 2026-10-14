//! `backfill_audit_event_decision_id` reads
//! `acme.audit.v1.AuditEventValidationService/ExplainAuditEventDecision` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::analytics::v1 as analytics;
use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::patientapp::v1 as patientapp;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        audit::audit_event_validation_service_client::AuditEventValidationServiceClient::new(channel.clone());
    let found = source
        .explain_audit_event_decision(audit::ExplainAuditEventDecisionRequest {
            context: Some(context.clone()),
            audit_event_id: "main-backfill-audit-event-decision-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let audit_event = found.audit_event.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = AuditEventDecisionIdRecord {
        correlation_id: audit_event.correlation_id,
        reference_id: outcome.reference_id,
    };
    dispatch_audit_event_decision_id("patient-home", record, channel, &context).await
}

/// What `backfill_audit_event_decision_id` keeps of each ExplainAuditEventDecision response.
#[derive(Clone, Debug, Default)]
pub struct AuditEventDecisionIdRecord {
    pub correlation_id: String,
    pub reference_id: String,
}

/// Where `backfill_audit_event_decision_id` hands each record.
trait AuditEventDecisionIdWriter {
    fn write(&self, record: AuditEventDecisionIdRecord) -> Delivery;
}

/// Writes records to `acme.analytics.v1.AnalyticsEventService`.
struct AnalyticsEventSink {
    client: analytics::analytics_event_service_client::AnalyticsEventServiceClient<Channel>,
    context: common::RequestContext,
}

impl AuditEventDecisionIdWriter for AnalyticsEventSink {
    fn write(&self, record: AuditEventDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(analytics::SearchAnalyticsEventsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.reference_id,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_analytics_events(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.community.v1.ForumService`.
struct ForumSink {
    client: community::forum_service_client::ForumServiceClient<Channel>,
}

impl AuditEventDecisionIdWriter for ForumSink {
    fn write(&self, record: AuditEventDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_forums(community::ListForumsRequest {
                    filter: Some(community::list_forums_request::Filter {
                        query: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.EnvironmentLifecycleService`.
struct EnvironmentLifecycleSink {
    client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl AuditEventDecisionIdWriter for EnvironmentLifecycleSink {
    fn write(&self, record: AuditEventDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::ExpireEnvironmentRequest {
                    details: Some(configuration::expire_environment_request::Details {
                        reason_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.expire_environment(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
}

impl AuditEventDecisionIdWriter for PatientHomeSink {
    fn write(&self, record: AuditEventDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_patient_homes(patientapp::ListPatientHomesRequest {
                    filter: Some(patientapp::list_patient_homes_request::Filter {
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

/// Only logs, for kinds no service takes yet.
struct AuditEventDecisionIdLogOnly;

impl AuditEventDecisionIdWriter for AuditEventDecisionIdLogOnly {
    fn write(&self, record: AuditEventDecisionIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_audit_event_decision_id(
    kind: &str,
    record: AuditEventDecisionIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let analytics_event = AnalyticsEventSink {
        client: analytics::analytics_event_service_client::AnalyticsEventServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let forum = ForumSink {
        client: community::forum_service_client::ForumServiceClient::new(channel.clone()),
    };
    let environment_lifecycle = EnvironmentLifecycleSink {
        client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient::new(
            channel.clone(),
        ),
        context: context.clone(),
    };
    let patient_home = PatientHomeSink {
        client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel),
    };
    let write: &dyn Fn(AuditEventDecisionIdRecord) -> Delivery = match kind {
        "analytics-event" => &|record| analytics_event.write(record),
        "forum" => &|record| forum.write(record),
        "environment-lifecycle" => &|record| environment_lifecycle.write(record),
        "patient-home" => &|record| patient_home.write(record),
        _ => &|record| AuditEventDecisionIdLogOnly.write(record),
    };
    write(record).await
}
