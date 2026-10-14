//! `hand_off_sponsor_message` reads `acme.sponsorships.v1.SponsorCalculationService/EstimateSponsor` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(channel.clone());
    let fetched = source
        .estimate_sponsor(sponsorships::EstimateSponsorRequest {
            context: Some(context.clone()),
            reason: "current-hand-off-sponsor-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let sponsor = fetched.sponsor.unwrap_or_default();
    let breakdown = fetched.breakdown.unwrap_or_default();
    let record = OffSponsorMessageRecord {
        message: result.message,
        sponsor_id: sponsor.sponsor_id,
        currency_code: breakdown.currency_code,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffSponsorMessageTarget>> = BTreeMap::new();
    sinks.insert(
        "throttle-calculation",
        Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "throttle-reporting",
        Box::new(ThrottleReportingSink {
            client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "csr-validation",
        Box::new(CsrValidationSink {
            client: certificates::csr_validation_service_client::CsrValidationServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("throttle-calculation") else {
        anyhow::bail!("no sink registered for throttle-calculation");
    };
    sink.apply(record).await
}

/// What `hand_off_sponsor_message` keeps of each EstimateSponsor response.
#[derive(Clone, Debug, Default)]
pub struct OffSponsorMessageRecord {
    pub message: String,
    pub sponsor_id: String,
    pub currency_code: String,
}

/// Where `hand_off_sponsor_message` hands each record.
trait OffSponsorMessageTarget {
    fn apply(&self, record: OffSponsorMessageRecord) -> Delivery;
}

/// Sends records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl OffSponsorMessageTarget for ThrottleCalculationSink {
    fn apply(&self, record: OffSponsorMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_throttle_options(ratelimiting::CompareThrottleOptionsRequest {
                    throttle_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleReportingService`.
struct ThrottleReportingSink {
    client: ratelimiting::throttle_reporting_service_client::ThrottleReportingServiceClient<Channel>,
}

impl OffSponsorMessageTarget for ThrottleReportingSink {
    fn apply(&self, record: OffSponsorMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_throttle_reports(ratelimiting::ListThrottleReportsRequest {
                    filter: Some(ratelimiting::list_throttle_reports_request::Filter {
                        query: record.message,
                        region: record.currency_code,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.sponsor_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CsrValidationService`.
struct CsrValidationSink {
    client: certificates::csr_validation_service_client::CsrValidationServiceClient<Channel>,
}

impl OffSponsorMessageTarget for CsrValidationSink {
    fn apply(&self, record: OffSponsorMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_csr(certificates::ScoreCsrRequest {
                    csr_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
