//! `hand_off_venue_eligibility_id` reads `acme.venues.v1.VenueValidationService/CheckVenueEligibility` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone());
    let found = source
        .check_venue_eligibility(venues::CheckVenueEligibilityRequest {
            context: Some(context.clone()),
            venue_id: "live-hand-off-venue-eligibility-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let venue = found.venue.unwrap_or_default();
    let record = OffVenueEligibilityIdRecord {
        reference_id: outcome.reference_id,
        name: venue.name,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffVenueEligibilityIdSink>> = BTreeMap::new();
    sinks.insert(
        "search-index-validation",
        Box::new(SearchIndexValidationSink {
            client: search::search_index_validation_service_client::SearchIndexValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "environment-approval",
        Box::new(EnvironmentApprovalSink {
            client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("environment-approval") else {
        anyhow::bail!("no sink registered for environment-approval");
    };
    sink.apply(record).await
}

/// What `hand_off_venue_eligibility_id` keeps of each CheckVenueEligibility response.
#[derive(Clone, Debug, Default)]
pub struct OffVenueEligibilityIdRecord {
    pub reference_id: String,
    pub name: String,
}

/// Where `hand_off_venue_eligibility_id` hands each record.
trait OffVenueEligibilityIdSink {
    fn apply(&self, record: OffVenueEligibilityIdRecord) -> Delivery;
}

/// Sends records to `acme.search.v1.SearchIndexValidationService`.
struct SearchIndexValidationSink {
    client: search::search_index_validation_service_client::SearchIndexValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffVenueEligibilityIdSink for SearchIndexValidationSink {
    fn apply(&self, record: OffVenueEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(search::ValidateSearchIndexRequest {
                    search_index: Some(search::SearchIndex {
                        code: record.name,
                        name: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.validate_search_index(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
}

impl OffVenueEligibilityIdSink for EnvironmentApprovalSink {
    fn apply(&self, record: OffVenueEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delegate_environment_approval(configuration::DelegateEnvironmentApprovalRequest {
                    reason: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
