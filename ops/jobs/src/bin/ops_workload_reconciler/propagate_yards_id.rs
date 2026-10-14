//! `propagate_yards_id` reads `acme.yards.v1.YardService/SearchYards` and hands what it keeps to the sink of
//! its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = yards::yard_service_client::YardServiceClient::new(channel.clone());
    let loaded = source
        .search_yards(yards::SearchYardsRequest {
            context: Some(context.clone()),
            order_by: "default-propagate-yards-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = YardsIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
    };
    let mut sinks: BTreeMap<&str, Box<dyn YardsIdDestination>> = BTreeMap::new();
    sinks.insert(
        "crypto-key-validation",
        Box::new(CryptoKeyValidationSink {
            client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "admission-calculation",
        Box::new(AdmissionCalculationSink {
            client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "key-ring-workflow",
        Box::new(KeyRingWorkflowSink {
            client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "feature-flag-validation",
        Box::new(FeatureFlagValidationSink {
            client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    );
    match sinks.get("admission-calculation") {
        Some(sink) => sink.forward(record).await,
        None => anyhow::bail!("no sink registered for admission-calculation"),
    }
}

/// What `propagate_yards_id` keeps of each SearchYards response.
#[derive(Clone, Debug, Default)]
pub struct YardsIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
}

/// Where `propagate_yards_id` hands each record.
trait YardsIdDestination {
    fn forward(&self, record: YardsIdRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyValidationService`.
struct CryptoKeyValidationSink {
    client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient<Channel>,
}

impl YardsIdDestination for CryptoKeyValidationSink {
    fn forward(&self, record: YardsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_crypto_key_rules(keymanagement::GetCryptoKeyRulesRequest {
                    crypto_key_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.admissions.v1.AdmissionCalculationService`.
struct AdmissionCalculationSink {
    client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient<Channel>,
}

impl YardsIdDestination for AdmissionCalculationSink {
    fn forward(&self, record: YardsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_admission_rates(admissions::GetAdmissionRatesRequest {
                    admission_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRingWorkflowService`.
struct KeyRingWorkflowSink {
    client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardsIdDestination for KeyRingWorkflowSink {
    fn forward(&self, record: YardsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::PauseKeyRingWorkflowRequest {
                    key_ring: Some(keymanagement::KeyRing {
                        parent_id: record.next_page_token,
                        ..Default::default()
                    }),
                    key_ring_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.pause_key_ring_workflow(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FeatureFlagValidationService`.
struct FeatureFlagValidationSink {
    client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardsIdDestination for FeatureFlagValidationSink {
    fn forward(&self, record: YardsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::LintFeatureFlagsRequest {
                    feature_flag: Some(featureflags::FeatureFlag {
                        name: record.next_page_token,
                        ..Default::default()
                    }),
                    details: Some(featureflags::lint_feature_flags_request::Details {
                        locale: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_feature_flags(request).await?;
            Ok(())
        })
    }
}
