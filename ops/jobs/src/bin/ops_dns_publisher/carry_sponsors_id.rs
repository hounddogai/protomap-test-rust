//! `carry_sponsors_id` reads `acme.sponsorships.v1.SponsorQueryService/CountSponsors` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone());
    let loaded = source
        .count_sponsors(sponsorships::CountSponsorsRequest {
            context: Some(context.clone()),
            order_by: "main-carry-sponsors-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = SponsorsIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        message: result.message,
    };
    let mut sinks: BTreeMap<String, Box<dyn SponsorsIdWriter>> = BTreeMap::new();
    sinks.insert(
        "crypto-key-query".to_owned(),
        Box::new(CryptoKeyQuerySink {
            client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "feature-flag-validation".to_owned(),
        Box::new(FeatureFlagValidationSink {
            client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "patient-home".to_owned(),
        Box::new(PatientHomeSink {
            client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel),
        }),
    );
    let target = std::env::var("CARRY_SPONSORS_ID_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.forward(record).await,
        None => Ok(()),
    }
}

/// What `carry_sponsors_id` keeps of each CountSponsors response.
#[derive(Clone, Debug, Default)]
pub struct SponsorsIdRecord {
    pub correlation_id: String,
    pub message: String,
}

/// Where `carry_sponsors_id` hands each record.
trait SponsorsIdWriter {
    fn forward(&self, record: SponsorsIdRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyQueryService`.
struct CryptoKeyQuerySink {
    client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient<Channel>,
}

impl SponsorsIdWriter for CryptoKeyQuerySink {
    fn forward(&self, record: SponsorsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_crypto_key(keymanagement::LookupCryptoKeyRequest {
                    crypto_key_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FeatureFlagValidationService`.
struct FeatureFlagValidationSink {
    client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SponsorsIdWriter for FeatureFlagValidationSink {
    fn forward(&self, record: SponsorsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::ScoreFeatureFlagRequest {
                    feature_flag: Some(featureflags::FeatureFlag {
                        feature_flag_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.score_feature_flag(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
}

impl SponsorsIdWriter for PatientHomeSink {
    fn forward(&self, record: SponsorsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_patient_home(patientapp::GetPatientHomeRequest {
                    patient_home_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
