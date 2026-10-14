//! `carry_config_set_rules_status` reads `acme.configuration.v1.ConfigSetValidationService/GetConfigSetRules`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_validation_service_client::ConfigSetValidationServiceClient::new(channel.clone());
    let fetched = source
        .get_config_set_rules(configuration::GetConfigSetRulesRequest {
            context: Some(context.clone()),
            config_set_id: "nightly-carry-config-set-rules-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let config_set = fetched.config_set.unwrap_or_default();
    let record = ConfigSetRulesStatusRecord {
        status: result.status,
        config_set_id: config_set.config_set_id,
        name: config_set.name,
        message: result.message,
    };
    let sinks: Vec<Box<dyn ConfigSetRulesStatusDestination>> = vec![
        Box::new(CryptoKeyCalculationSink {
            client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(SellerSink {
            client: sellers::seller_service_client::SellerServiceClient::new(channel),
        }),
    ];
    sinks[1].push(record).await
}

/// What `carry_config_set_rules_status` keeps of each GetConfigSetRules response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetRulesStatusRecord {
    pub status: String,
    pub config_set_id: String,
    pub name: String,
    pub message: String,
}

/// Where `carry_config_set_rules_status` hands each record.
trait ConfigSetRulesStatusDestination {
    fn push(&self, record: ConfigSetRulesStatusRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl ConfigSetRulesStatusDestination for CryptoKeyCalculationSink {
    fn push(&self, record: ConfigSetRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_crypto_key(keymanagement::SimulateCryptoKeyRequest {
                    crypto_key_id: record.config_set_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.sellers.v1.SellerService`.
struct SellerSink {
    client: sellers::seller_service_client::SellerServiceClient<Channel>,
}

impl ConfigSetRulesStatusDestination for SellerSink {
    fn push(&self, record: ConfigSetRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_sellers(sellers::SearchSellersRequest {
                    filter: Some(sellers::search_sellers_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
