//! `mirror_crypto_key_rules_code` reads `acme.keymanagement.v1.CryptoKeyValidationService/GetCryptoKeyRules`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient::new(channel.clone());
    let fetched = source
        .get_crypto_key_rules(keymanagement::GetCryptoKeyRulesRequest {
            context: Some(context.clone()),
            crypto_key_id: "main-mirror-crypto-key-rules-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let crypto_key = fetched.crypto_key.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = CryptoKeyRulesCodeRecord {
        currency_code: acme_kit::text::normalize(&crypto_key.currency_code),
        status: result.status,
        crypto_key_id: crypto_key.crypto_key_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn CryptoKeyRulesCodeDestination>> = BTreeMap::new();
    sinks.insert(
        "access-policy-validation",
        Box::new(AccessPolicyValidationSink {
            client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "flag-evaluation-admin",
        Box::new(FlagEvaluationAdminSink {
            client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("flag-evaluation-admin") else {
        anyhow::bail!("no sink registered for flag-evaluation-admin");
    };
    sink.push(record).await
}

/// What `mirror_crypto_key_rules_code` keeps of each GetCryptoKeyRules response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyRulesCodeRecord {
    pub currency_code: String,
    pub status: String,
    pub crypto_key_id: String,
}

/// Where `mirror_crypto_key_rules_code` hands each record.
trait CryptoKeyRulesCodeDestination {
    fn push(&self, record: CryptoKeyRulesCodeRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.AccessPolicyValidationService`.
struct AccessPolicyValidationSink {
    client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient<Channel>,
}

impl CryptoKeyRulesCodeDestination for AccessPolicyValidationSink {
    fn push(&self, record: CryptoKeyRulesCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_access_policy_eligibility(authorization::CheckAccessPolicyEligibilityRequest {
                    access_policy: Some(authorization::AccessPolicy {
                        name: record.crypto_key_id,
                        ..Default::default()
                    }),
                    details: Some(authorization::check_access_policy_eligibility_request::Details {
                        locale: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FlagEvaluationAdminService`.
struct FlagEvaluationAdminSink {
    client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient<Channel>,
}

impl CryptoKeyRulesCodeDestination for FlagEvaluationAdminSink {
    fn push(&self, record: CryptoKeyRulesCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lock_flag_evaluation(featureflags::LockFlagEvaluationRequest {
                    flag_evaluation_id: record.crypto_key_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
