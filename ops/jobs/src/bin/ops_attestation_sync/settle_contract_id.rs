//! `settle_contract_id` reads `acme.contracts.v1.ContractService/GetContract` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::contracts::v1 as contracts;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::loyalty::v1 as loyalty;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = contracts::contract_service_client::ContractServiceClient::new(channel.clone());
    let loaded = source
        .get_contract(contracts::GetContractRequest {
            context: Some(context.clone()),
            contract_id: "default-settle-contract-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let contract = loaded.contract.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = ContractIdRecord {
        contract_id: contract.contract_id,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    dispatch_contract_id("show-calculation", record, channel, &context).await
}

/// What `settle_contract_id` keeps of each GetContract response.
#[derive(Clone, Debug, Default)]
pub struct ContractIdRecord {
    pub contract_id: String,
    pub correlation_id: String,
}

/// Where `settle_contract_id` hands each record.
trait ContractIdDestination {
    fn forward(&self, record: ContractIdRecord) -> Delivery;
}

/// Passes records to `acme.identity.v1.UserService`.
struct UserSink {
    client: identity::user_service_client::UserServiceClient<Channel>,
}

impl ContractIdDestination for UserSink {
    fn forward(&self, record: ContractIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_user(identity::UpdateUserRequest {
                    user: Some(identity::User {
                        user_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.shows.v1.ShowCalculationService`.
struct ShowCalculationSink {
    client: shows::show_calculation_service_client::ShowCalculationServiceClient<Channel>,
}

impl ContractIdDestination for ShowCalculationSink {
    fn forward(&self, record: ContractIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_show_rates(shows::GetShowRatesRequest {
                    show_id: record.contract_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.loyalty.v1.LoyaltyMemberQueryService`.
struct LoyaltyMemberQuerySink {
    client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient<Channel>,
}

impl ContractIdDestination for LoyaltyMemberQuerySink {
    fn forward(&self, record: ContractIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_loyalty_members(loyalty::FindLoyaltyMembersRequest {
                    order_by: record.contract_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ContractIdLogOnly;

impl ContractIdDestination for ContractIdLogOnly {
    fn forward(&self, record: ContractIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.contract_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_contract_id(
    kind: &str,
    record: ContractIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let user = UserSink {
        client: identity::user_service_client::UserServiceClient::new(channel.clone()),
    };
    let show_calculation = ShowCalculationSink {
        client: shows::show_calculation_service_client::ShowCalculationServiceClient::new(channel.clone()),
    };
    let loyalty_member_query = LoyaltyMemberQuerySink {
        client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient::new(channel),
    };
    let write: &dyn Fn(ContractIdRecord) -> Delivery = match kind {
        "user" => &|record| user.forward(record),
        "show-calculation" => &|record| show_calculation.forward(record),
        "loyalty-member-query" => &|record| loyalty_member_query.forward(record),
        _ => &|record| ContractIdLogOnly.forward(record),
    };
    write(record).await
}
