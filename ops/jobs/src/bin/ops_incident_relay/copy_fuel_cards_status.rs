//! `copy_fuel_cards_status` reads `acme.fuel.v1.FuelCardQueryService/CountFuelCards` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::contracts::v1 as contracts;
use acme_proto::acme::fuel::v1 as fuel;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fuel::fuel_card_query_service_client::FuelCardQueryServiceClient::new(channel.clone());
    let fetched = source
        .count_fuel_cards(fuel::CountFuelCardsRequest {
            context: Some(context.clone()),
            order_by: "primary-copy-fuel-cards-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = FuelCardsStatusRecord {
        status: result.status,
        message: result.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    let sinks: Vec<Box<dyn FuelCardsStatusWriter>> = vec![
        Box::new(AccessGrantApprovalSink {
            client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(CryptoKeyCalculationSink {
            client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(AccessPolicyHistorySink {
            client: authorization::access_policy_history_service_client::AccessPolicyHistoryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(ContractSink {
            client: contracts::contract_service_client::ContractServiceClient::new(channel),
        }),
    ];
    sinks[0].forward(record).await
}

/// What `copy_fuel_cards_status` keeps of each CountFuelCards response.
#[derive(Clone, Debug, Default)]
pub struct FuelCardsStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `copy_fuel_cards_status` hands each record.
trait FuelCardsStatusWriter {
    fn forward(&self, record: FuelCardsStatusRecord) -> Delivery;
}

/// Hands records to `acme.secrets.v1.AccessGrantApprovalService`.
struct AccessGrantApprovalSink {
    client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl FuelCardsStatusWriter for AccessGrantApprovalSink {
    fn forward(&self, record: FuelCardsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::ApproveAccessGrantRequest {
                    access_grant: Some(secrets::AccessGrant {
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.message,
                    access_grant_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.approve_access_grant(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl FuelCardsStatusWriter for CryptoKeyCalculationSink {
    fn forward(&self, record: FuelCardsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::CalculateCryptoKeyRequest {
                    reason: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_crypto_key(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.AccessPolicyHistoryService`.
struct AccessPolicyHistorySink {
    client: authorization::access_policy_history_service_client::AccessPolicyHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FuelCardsStatusWriter for AccessPolicyHistorySink {
    fn forward(&self, record: FuelCardsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::CompareAccessPolicyVersionsRequest {
                    reason: record.correlation_id,
                    access_policy: Some(authorization::AccessPolicy {
                        access_policy_id: record.message,
                        description: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.compare_access_policy_versions(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.contracts.v1.ContractService`.
struct ContractSink {
    client: contracts::contract_service_client::ContractServiceClient<Channel>,
}

impl FuelCardsStatusWriter for ContractSink {
    fn forward(&self, record: FuelCardsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_contract(contracts::UpdateContractRequest {
                    contract: Some(contracts::Contract {
                        contract_id: record.status,
                        number: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
