//! `hand_off_analytics_event_id` reads `acme.analytics.v1.AnalyticsEventService/GetAnalyticsEvent` and hands
//! what it keeps to `acme.configuration.v1.ConfigChangeQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::analytics::v1 as analytics;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = analytics::analytics_event_service_client::AnalyticsEventServiceClient::new(channel.clone());
    let fetched = source
        .get_analytics_event(analytics::GetAnalyticsEventRequest {
            context: Some(context.clone()),
            analytics_event_id: "primary-hand-off-analytics-event-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let analytics_event = fetched.analytics_event.unwrap_or_default();
    let record = OffAnalyticsEventIdRecord {
        correlation_id: result.correlation_id,
        kind: analytics_event.kind,
        source: analytics_event.source,
        message: acme_kit::text::normalize(&result.message),
    };
    ConfigChangeQuerySink {
        client: configuration::config_change_query_service_client::ConfigChangeQueryServiceClient::new(channel),
    }
    .push(record)
    .await
}

/// What `hand_off_analytics_event_id` keeps of each GetAnalyticsEvent response.
#[derive(Clone, Debug, Default)]
pub struct OffAnalyticsEventIdRecord {
    pub correlation_id: String,
    pub kind: String,
    pub source: String,
    pub message: String,
}

/// Where `hand_off_analytics_event_id` hands each record.
trait OffAnalyticsEventIdWriter {
    fn push(&self, record: OffAnalyticsEventIdRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.ConfigChangeQueryService`.
struct ConfigChangeQuerySink {
    client: configuration::config_change_query_service_client::ConfigChangeQueryServiceClient<Channel>,
}

impl OffAnalyticsEventIdWriter for ConfigChangeQuerySink {
    fn push(&self, record: OffAnalyticsEventIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_config_changes(configuration::BrowseConfigChangesRequest {
                    order_by: record.kind,
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    filter: Some(configuration::browse_config_changes_request::Filter {
                        region: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
