//! `route_timesheets_id` reads `acme.timekeeping.v1.TimesheetQueryService/CountTimesheets` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::lineage::v1 as lineage;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::timekeeping::v1 as timekeeping;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = timekeeping::timesheet_query_service_client::TimesheetQueryServiceClient::new(channel.clone());
    let fetched = source
        .count_timesheets(timekeeping::CountTimesheetsRequest {
            context: Some(context.clone()),
            order_by: "live-route-timesheets-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = TimesheetsIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn TimesheetsIdSink>)> = vec![
        (
            "throttle-reporting",
            Box::new(ThrottleReportingSink {
                client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "lineage-node-query",
            Box::new(LineageNodeQuerySink {
                client: lineage::lineage_node_query_service_client::LineageNodeQueryServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "lineage-node-query" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `route_timesheets_id` keeps of each CountTimesheets response.
#[derive(Clone, Debug, Default)]
pub struct TimesheetsIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub status: String,
}

/// Where `route_timesheets_id` hands each record.
trait TimesheetsIdSink {
    fn push(&self, record: TimesheetsIdRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.ThrottleReportingService`.
struct ThrottleReportingSink {
    client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient<Channel>,
}

impl TimesheetsIdSink for ThrottleReportingSink {
    fn push(&self, record: TimesheetsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_throttle_reports(ratelimiting::ListThrottleReportsRequest {
                    filter: Some(ratelimiting::list_throttle_reports_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.lineage.v1.LineageNodeQueryService`.
struct LineageNodeQuerySink {
    client: lineage::lineage_node_query_service_client::LineageNodeQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl TimesheetsIdSink for LineageNodeQuerySink {
    fn push(&self, record: TimesheetsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(lineage::SummarizeLineageNodesRequest {
                    details: Some(lineage::summarize_lineage_nodes_request::Details {
                        format: record.status,
                        cron: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_lineage_nodes(request).await?;
            Ok(())
        })
    }
}
