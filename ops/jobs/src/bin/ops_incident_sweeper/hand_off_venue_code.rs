//! `hand_off_venue_code` reads `acme.venues.v1.VenueValidationService/ScoreVenue` and hands what it keeps to
//! `acme.configuration.v1.ConfigChangeReportingService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone());
    let found = source
        .score_venue(venues::ScoreVenueRequest {
            context: Some(context.clone()),
            reason: "main-hand-off-venue-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = OffVenueCodeRecord {
        currency_code: breakdown.currency_code,
        status: result.status,
        correlation_id: result.correlation_id,
    };
    ConfigChangeReportingSink {
        client: configuration::config_change_reporting_service_client::ConfigChangeReportingServiceClient::new(channel),
    }
    .write(record)
    .await
}

/// What `hand_off_venue_code` keeps of each ScoreVenue response.
#[derive(Clone, Debug, Default)]
pub struct OffVenueCodeRecord {
    pub currency_code: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `hand_off_venue_code` hands each record.
trait OffVenueCodeDestination {
    fn write(&self, record: OffVenueCodeRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.ConfigChangeReportingService`.
struct ConfigChangeReportingSink {
    client: configuration::config_change_reporting_service_client::ConfigChangeReportingServiceClient<Channel>,
}

impl OffVenueCodeDestination for ConfigChangeReportingSink {
    fn write(&self, record: OffVenueCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .generate_config_change_report(configuration::GenerateConfigChangeReportRequest {
                    reason: record.currency_code,
                    details: Some(configuration::generate_config_change_report_request::Details {
                        cron: record.correlation_id,
                        format: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
