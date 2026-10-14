//! `push_cash_position_message` reads `acme.treasury.v1.CashPositionCalculationService/CalculateCashPosition`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::telehealth::v1 as telehealth;
use acme_proto::acme::treasury::v1 as treasury;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient::new(channel.clone());
    let reply = source
        .calculate_cash_position(treasury::CalculateCashPositionRequest {
            context: Some(context.clone()),
            reason: "live-push-cash-position-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let breakdown = reply.breakdown.unwrap_or_default();
    let record = CashPositionMessageRecord {
        message: result.message,
        currency_code: breakdown.currency_code,
    };
    let mut sinks: BTreeMap<&str, Box<dyn CashPositionMessageTarget>> = BTreeMap::new();
    sinks.insert(
        "secret-version-validation",
        Box::new(SecretVersionValidationSink {
            client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "video-visit-calculation",
        Box::new(VideoVisitCalculationSink {
            client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "venue",
        Box::new(VenueSink {
            client: venues::venue_service_client::VenueServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("video-visit-calculation") else {
        anyhow::bail!("no sink registered for video-visit-calculation");
    };
    sink.deliver(record).await
}

/// What `push_cash_position_message` keeps of each CalculateCashPosition response.
#[derive(Clone, Debug, Default)]
pub struct CashPositionMessageRecord {
    pub message: String,
    pub currency_code: String,
}

/// Where `push_cash_position_message` hands each record.
trait CashPositionMessageTarget {
    fn deliver(&self, record: CashPositionMessageRecord) -> Delivery;
}

/// Hands records to `acme.secrets.v1.SecretVersionValidationService`.
struct SecretVersionValidationSink {
    client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient<Channel>,
}

impl CashPositionMessageTarget for SecretVersionValidationSink {
    fn deliver(&self, record: CashPositionMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_secret_versions(secrets::LintSecretVersionsRequest {
                    secret_version: Some(secrets::SecretVersion {
                        kind: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.telehealth.v1.VideoVisitCalculationService`.
struct VideoVisitCalculationSink {
    client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CashPositionMessageTarget for VideoVisitCalculationSink {
    fn deliver(&self, record: CashPositionMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(telehealth::SimulateVideoVisitRequest {
                    details: Some(telehealth::simulate_video_visit_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.simulate_video_visit(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.venues.v1.VenueService`.
struct VenueSink {
    client: venues::venue_service_client::VenueServiceClient<Channel>,
}

impl CashPositionMessageTarget for VenueSink {
    fn deliver(&self, record: CashPositionMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_venue(venues::CreateVenueRequest {
                    request_id: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
