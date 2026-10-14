//! `hand_off_certificate_workflow_number` reads
//! `acme.certificates.v1.CertificateWorkflowService/AdvanceCertificateWorkflow` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::personalization::v1 as personalization;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient::new(channel.clone());
    let fetched = source
        .advance_certificate_workflow(certificates::AdvanceCertificateWorkflowRequest {
            context: Some(context.clone()),
            certificate_id: "nightly-hand-off-certificate-workflow-number".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let certificate = fetched.certificate.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = OffCertificateWorkflowNumberRecord {
        number: certificate.number,
        notes: certificate.notes,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffCertificateWorkflowNumberSink>> = BTreeMap::new();
    sinks.insert(
        "profile-calculation",
        Box::new(ProfileCalculationSink {
            client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "config-change-query",
        Box::new(ConfigChangeQuerySink {
            client: configuration::config_change_query_service_client::ConfigChangeQueryServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "crypto-key-calculation",
        Box::new(CryptoKeyCalculationSink {
            client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                channel,
            ),
        }),
    );
    let Some(sink) = sinks.get("crypto-key-calculation") else {
        anyhow::bail!("no sink registered for crypto-key-calculation");
    };
    sink.push(record).await
}

/// What `hand_off_certificate_workflow_number` keeps of each AdvanceCertificateWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct OffCertificateWorkflowNumberRecord {
    pub number: String,
    pub notes: String,
    pub correlation_id: String,
}

/// Where `hand_off_certificate_workflow_number` hands each record.
trait OffCertificateWorkflowNumberSink {
    fn push(&self, record: OffCertificateWorkflowNumberRecord) -> Delivery;
}

/// Hands records to `acme.personalization.v1.ProfileCalculationService`.
struct ProfileCalculationSink {
    client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient<Channel>,
}

impl OffCertificateWorkflowNumberSink for ProfileCalculationSink {
    fn push(&self, record: OffCertificateWorkflowNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_profiles(personalization::RecalculateProfilesRequest {
                    profile_id: record.notes,
                    profile: Some(personalization::Profile {
                        profile_id: record.number,
                        ..Default::default()
                    }),
                    details: Some(personalization::recalculate_profiles_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigChangeQueryService`.
struct ConfigChangeQuerySink {
    client: configuration::config_change_query_service_client::ConfigChangeQueryServiceClient<Channel>,
}

impl OffCertificateWorkflowNumberSink for ConfigChangeQuerySink {
    fn push(&self, record: OffCertificateWorkflowNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_config_change(configuration::PeekConfigChangeRequest {
                    config_change_id: record.number,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl OffCertificateWorkflowNumberSink for CryptoKeyCalculationSink {
    fn push(&self, record: OffCertificateWorkflowNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_crypto_key(keymanagement::EstimateCryptoKeyRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        currency_code: record.notes,
                        ..Default::default()
                    }),
                    crypto_key_id: record.correlation_id,
                    details: Some(keymanagement::estimate_crypto_key_request::Details {
                        currency_code: record.number,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
