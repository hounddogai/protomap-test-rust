//! `forward_fan_session_id` reads `acme.fanportal.v1.FanSessionQueryService/LookupFanSession` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::loyalty::v1 as loyalty;
use acme_proto::acme::timekeeping::v1 as timekeeping;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fanportal::fan_session_query_service_client::FanSessionQueryServiceClient::new(channel.clone());
    let reply = source
        .lookup_fan_session(fanportal::LookupFanSessionRequest {
            context: Some(context.clone()),
            fan_session_id: "default-forward-fan-session-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let fan_session = reply.fan_session.unwrap_or_default();
    let record = FanSessionIdRecord {
        correlation_id: result.correlation_id,
        message: acme_kit::text::normalize(&result.message),
        fan_session_id: acme_kit::text::normalize(&fan_session.fan_session_id),
    };
    let sinks: Vec<(&str, Box<dyn FanSessionIdSink>)> = vec![
        (
            "timesheet-query",
            Box::new(TimesheetQuerySink {
                client: timekeeping::timesheet_query_service_client::TimesheetQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "csr-workflow",
            Box::new(CsrWorkflowSink {
                client: certificates::csr_workflow_service_client::CsrWorkflowServiceClient::new(channel.clone()),
            }),
        ),
        (
            "loyalty-member-query",
            Box::new(LoyaltyMemberQuerySink {
                client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "timesheet-query" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `forward_fan_session_id` keeps of each LookupFanSession response.
#[derive(Clone, Debug, Default)]
pub struct FanSessionIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub fan_session_id: String,
}

/// Where `forward_fan_session_id` hands each record.
trait FanSessionIdSink {
    fn apply(&self, record: FanSessionIdRecord) -> Delivery;
}

/// Sends records to `acme.timekeeping.v1.TimesheetQueryService`.
struct TimesheetQuerySink {
    client: timekeeping::timesheet_query_service_client::TimesheetQueryServiceClient<Channel>,
}

impl FanSessionIdSink for TimesheetQuerySink {
    fn apply(&self, record: FanSessionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_timesheets(timekeeping::CountTimesheetsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    filter: Some(timekeeping::count_timesheets_request::Filter {
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

/// Passes records to `acme.certificates.v1.CsrWorkflowService`.
struct CsrWorkflowSink {
    client: certificates::csr_workflow_service_client::CsrWorkflowServiceClient<Channel>,
}

impl FanSessionIdSink for CsrWorkflowSink {
    fn apply(&self, record: FanSessionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_csr_workflow(certificates::RetryCsrWorkflowRequest {
                    csr_id: record.fan_session_id,
                    reason: record.message,
                    csr: Some(certificates::Csr {
                        subject: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.loyalty.v1.LoyaltyMemberQueryService`.
struct LoyaltyMemberQuerySink {
    client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FanSessionIdSink for LoyaltyMemberQuerySink {
    fn apply(&self, record: FanSessionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(loyalty::LookupLoyaltyMemberRequest {
                    loyalty_member_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_loyalty_member(request).await?;
            Ok(())
        })
    }
}
