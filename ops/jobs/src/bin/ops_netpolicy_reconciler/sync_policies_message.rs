//! `sync_policies_message` reads `acme.insurance.v1.PolicyService/ListPolicies` and hands what it keeps to
//! `acme.fuel.v1.FuelCardQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fuel::v1 as fuel;
use acme_proto::acme::insurance::v1 as insurance;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = insurance::policy_service_client::PolicyServiceClient::new(channel.clone());
    let reply = source
        .list_policies(insurance::ListPoliciesRequest {
            context: Some(context.clone()),
            order_by: "primary-sync-policies-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = PoliciesMessageRecord {
        message: result.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
    };
    FuelCardQuerySink {
        client: fuel::fuel_card_query_service_client::FuelCardQueryServiceClient::new(channel),
    }
    .send(record)
    .await
}

/// What `sync_policies_message` keeps of each ListPolicies response.
#[derive(Clone, Debug, Default)]
pub struct PoliciesMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub next_page_token: String,
}

/// Where `sync_policies_message` hands each record.
trait PoliciesMessageDestination {
    fn send(&self, record: PoliciesMessageRecord) -> Delivery;
}

/// Passes records to `acme.fuel.v1.FuelCardQueryService`.
struct FuelCardQuerySink {
    client: fuel::fuel_card_query_service_client::FuelCardQueryServiceClient<Channel>,
}

impl PoliciesMessageDestination for FuelCardQuerySink {
    fn send(&self, record: PoliciesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_fuel_cards(fuel::FindFuelCardsRequest {
                    filter: Some(fuel::find_fuel_cards_request::Filter {
                        query: record.next_page_token,
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
