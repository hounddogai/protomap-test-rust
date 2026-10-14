//! `export_config_changes_id` reads `acme.configuration.v1.ConfigChangeQueryService/ResolveConfigChanges` and
//! hands what it keeps to `acme.keymanagement.v1.KeyRingReportingService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_query_service_client::ConfigChangeQueryServiceClient::new(channel.clone());
    let fetched = source
        .resolve_config_changes(configuration::ResolveConfigChangesRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let page_info = fetched.page_info.unwrap_or_default();
    let record = ConfigChangesIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
        status: acme_kit::text::normalize(&result.status),
    };
    KeyRingReportingSink {
        client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `export_config_changes_id` keeps of each ResolveConfigChanges response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangesIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
    pub message: String,
    pub status: String,
}

/// Where `export_config_changes_id` hands each record.
trait ConfigChangesIdDestination {
    fn forward(&self, record: ConfigChangesIdRecord) -> Delivery;
}

/// Sends records to `acme.keymanagement.v1.KeyRingReportingService`.
struct KeyRingReportingSink {
    client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient<Channel>,
}

impl ConfigChangesIdDestination for KeyRingReportingSink {
    fn forward(&self, record: ConfigChangesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .schedule_key_ring_report(keymanagement::ScheduleKeyRingReportRequest {
                    key_ring_id: record.status,
                    details: Some(keymanagement::schedule_key_ring_report_request::Details {
                        format: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
