//! `copy_venue_decision_id` reads `acme.venues.v1.VenueValidationService/ExplainVenueDecision` and hands what
//! it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone());
    let found = source
        .explain_venue_decision(venues::ExplainVenueDecisionRequest {
            context: Some(context.clone()),
            reason: "live-copy-venue-decision-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let venue = found.venue.unwrap_or_default();
    let record = VenueDecisionIdRecord {
        reference_id: outcome.reference_id,
        message: outcome.message,
        name: venue.name,
    };
    act_on_venue_decision_id("role-binding-admin", record, channel, context).await
}

/// What `copy_venue_decision_id` keeps of each ExplainVenueDecision response.
#[derive(Clone, Debug, Default)]
pub struct VenueDecisionIdRecord {
    pub reference_id: String,
    pub message: String,
    pub name: String,
}

/// Where `copy_venue_decision_id` hands each record.
trait VenueDecisionIdDestination {
    fn send(&self, record: VenueDecisionIdRecord) -> Delivery;
}

/// Sends records to `acme.search.v1.SearchIndexValidationService`.
struct SearchIndexValidationSink {
    client: search::search_index_validation_service_client::SearchIndexValidationServiceClient<Channel>,
}

impl VenueDecisionIdDestination for SearchIndexValidationSink {
    fn send(&self, record: VenueDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_search_index(search::PreviewSearchIndexRequest {
                    reason: record.reference_id,
                    details: Some(search::preview_search_index_request::Details {
                        locale: record.message,
                        ..Default::default()
                    }),
                    search_index: Some(search::SearchIndex {
                        name: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.RoleBindingAdminService`.
struct RoleBindingAdminSink {
    client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient<Channel>,
}

impl VenueDecisionIdDestination for RoleBindingAdminSink {
    fn send(&self, record: VenueDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .repair_role_binding(authorization::RepairRoleBindingRequest {
                    details: Some(authorization::repair_role_binding_request::Details {
                        resolution: record.message,
                        ..Default::default()
                    }),
                    role_binding: Some(authorization::RoleBinding {
                        number: record.name,
                        ..Default::default()
                    }),
                    role_binding_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_venue_decision_id(
    kind: &str,
    record: VenueDecisionIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(VenueDecisionIdRecord) -> Delivery>> = BTreeMap::new();
    let search_index_validation_channel = channel.clone();
    actions.insert(
        "search-index-validation",
        Box::new(move |record| {
            SearchIndexValidationSink {
                client: search::search_index_validation_service_client::SearchIndexValidationServiceClient::new(
                    search_index_validation_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    let role_binding_admin_channel = channel.clone();
    actions.insert(
        "role-binding-admin",
        Box::new(move |record| {
            RoleBindingAdminSink {
                client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient::new(
                    role_binding_admin_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
