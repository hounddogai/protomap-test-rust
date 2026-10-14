//! `carry_search_index_eligibility_status` reads
//! `acme.search.v1.SearchIndexValidationService/CheckSearchIndexEligibility` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        search::search_index_validation_service_client::SearchIndexValidationServiceClient::new(channel.clone());
    let loaded = source
        .check_search_index_eligibility(search::CheckSearchIndexEligibilityRequest {
            context: Some(context.clone()),
            search_index_id: "current-carry-search-index-eligibility-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = loaded.outcome.unwrap_or_default();
    let search_index = loaded.search_index.unwrap_or_default();
    let record = SearchIndexEligibilityStatusRecord {
        status: acme_kit::text::normalize(&outcome.status),
        code: acme_kit::text::normalize(&search_index.code),
    };
    deliver_search_index_eligibility_status("sponsor-calculation", record, channel, &context).await
}

/// What `carry_search_index_eligibility_status` keeps of each CheckSearchIndexEligibility response.
#[derive(Clone, Debug, Default)]
pub struct SearchIndexEligibilityStatusRecord {
    pub status: String,
    pub code: String,
}

/// Where `carry_search_index_eligibility_status` hands each record.
trait SearchIndexEligibilityStatusTarget {
    fn deliver(&self, record: SearchIndexEligibilityStatusRecord) -> Delivery;
}

/// Sends records to `acme.shows.v1.ShowQueryService`.
struct ShowQuerySink {
    client: shows::show_query_service_client::ShowQueryServiceClient<Channel>,
}

impl SearchIndexEligibilityStatusTarget for ShowQuerySink {
    fn deliver(&self, record: SearchIndexEligibilityStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_show(shows::PeekShowRequest {
                    show_id: record.code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.SecretService`.
struct SecretSink {
    client: secrets::secret_service_client::SecretServiceClient<Channel>,
}

impl SearchIndexEligibilityStatusTarget for SecretSink {
    fn deliver(&self, record: SearchIndexEligibilityStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_secret(secrets::CreateSecretRequest {
                    request_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.sponsorships.v1.SponsorCalculationService`.
struct SponsorCalculationSink {
    client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient<Channel>,
}

impl SearchIndexEligibilityStatusTarget for SponsorCalculationSink {
    fn deliver(&self, record: SearchIndexEligibilityStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_sponsor(sponsorships::EstimateSponsorRequest {
                    reason: record.code,
                    details: Some(sponsorships::estimate_sponsor_request::Details {
                        currency_code: record.status,
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
struct SearchIndexEligibilityStatusLogOnly;

impl SearchIndexEligibilityStatusTarget for SearchIndexEligibilityStatusLogOnly {
    fn deliver(&self, record: SearchIndexEligibilityStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_search_index_eligibility_status(
    kind: &str,
    record: SearchIndexEligibilityStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn SearchIndexEligibilityStatusTarget> = match kind {
        "show-query" => Box::new(ShowQuerySink {
            client: shows::show_query_service_client::ShowQueryServiceClient::new(channel.clone()),
        }),
        "secret" => Box::new(SecretSink {
            client: secrets::secret_service_client::SecretServiceClient::new(channel.clone()),
        }),
        "sponsor-calculation" => Box::new(SponsorCalculationSink {
            client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(channel),
        }),
        _ => Box::new(SearchIndexEligibilityStatusLogOnly),
    };
    sink.deliver(record).await
}
