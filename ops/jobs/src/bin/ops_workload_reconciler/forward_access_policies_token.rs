//! `forward_access_policies_token` reads `acme.authorization.v1.AccessPolicyService/ListAccessPolicies` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = authorization::access_policy_service_client::AccessPolicyServiceClient::new(channel.clone());
    let loaded = source
        .list_access_policies(authorization::ListAccessPoliciesRequest {
            context: Some(context.clone()),
            order_by: "primary-forward-access-policies-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = loaded.page_info.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = AccessPoliciesTokenRecord {
        next_page_token: page_info.next_page_token,
        status: result.status,
    };
    let sinks: Vec<Box<dyn AccessPoliciesTokenWriter>> = vec![
        Box::new(FulfillmentSink {
            client: fulfillment::fulfillment_service_client::FulfillmentServiceClient::new(channel.clone()),
        }),
        Box::new(CryptoKeyValidationSink {
            client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(ExperimentCalculationSink {
            client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(ApiKeySink {
            client: edgegateway::api_key_service_client::ApiKeyServiceClient::new(channel),
        }),
    ];
    sinks[2].write(record).await
}

/// What `forward_access_policies_token` keeps of each ListAccessPolicies response.
#[derive(Clone, Debug, Default)]
pub struct AccessPoliciesTokenRecord {
    pub next_page_token: String,
    pub status: String,
}

/// Where `forward_access_policies_token` hands each record.
trait AccessPoliciesTokenWriter {
    fn write(&self, record: AccessPoliciesTokenRecord) -> Delivery;
}

/// Passes records to `acme.fulfillment.v1.FulfillmentService`.
struct FulfillmentSink {
    client: fulfillment::fulfillment_service_client::FulfillmentServiceClient<Channel>,
}

impl AccessPoliciesTokenWriter for FulfillmentSink {
    fn write(&self, record: AccessPoliciesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_fulfillment(fulfillment::UpdateFulfillmentRequest {
                    fulfillment: Some(fulfillment::Fulfillment {
                        notes: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyValidationService`.
struct CryptoKeyValidationSink {
    client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient<Channel>,
}

impl AccessPoliciesTokenWriter for CryptoKeyValidationSink {
    fn write(&self, record: AccessPoliciesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_crypto_key_decision(keymanagement::ExplainCryptoKeyDecisionRequest {
                    crypto_key_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
}

impl AccessPoliciesTokenWriter for ExperimentCalculationSink {
    fn write(&self, record: AccessPoliciesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_experiment_options(experiments::CompareExperimentOptionsRequest {
                    experiment: Some(experiments::Experiment {
                        title: record.status,
                        experiment_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.ApiKeyService`.
struct ApiKeySink {
    client: edgegateway::api_key_service_client::ApiKeyServiceClient<Channel>,
}

impl AccessPoliciesTokenWriter for ApiKeySink {
    fn write(&self, record: AccessPoliciesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_api_key(edgegateway::UpdateApiKeyRequest {
                    api_key: Some(edgegateway::ApiKey {
                        currency_code: record.status,
                        api_key_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
