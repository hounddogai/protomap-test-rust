//! `export_crypto_key_status` reads `acme.keymanagement.v1.CryptoKeyLifecycleService/RenewCryptoKey` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(channel.clone());
    let fetched = source
        .renew_crypto_key(keymanagement::RenewCryptoKeyRequest {
            context: Some(context.clone()),
            crypto_key_id: "main-export-crypto-key-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let crypto_key = fetched.crypto_key.unwrap_or_default();
    let record = CryptoKeyStatusRecord {
        status: result.status,
        crypto_key_id: acme_kit::text::normalize(&crypto_key.crypto_key_id),
    };
    let mut sinks: BTreeMap<&str, Box<dyn CryptoKeyStatusTarget>> = BTreeMap::new();
    sinks.insert(
        "rate-limit-lifecycle",
        Box::new(RateLimitLifecycleSink {
            client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "patient-home-query",
        Box::new(PatientHomeQuerySink {
            client: patientapp::patient_home_query_service_client::PatientHomeQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("patient-home-query") {
        Some(sink) => sink.deliver(record).await,
        None => anyhow::bail!("no sink registered for patient-home-query"),
    }
}

/// What `export_crypto_key_status` keeps of each RenewCryptoKey response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyStatusRecord {
    pub status: String,
    pub crypto_key_id: String,
}

/// Where `export_crypto_key_status` hands each record.
trait CryptoKeyStatusTarget {
    fn deliver(&self, record: CryptoKeyStatusRecord) -> Delivery;
}

/// Sends records to `acme.ratelimiting.v1.RateLimitLifecycleService`.
struct RateLimitLifecycleSink {
    client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyStatusTarget for RateLimitLifecycleSink {
    fn deliver(&self, record: CryptoKeyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::GetRateLimitLifecycleRequest {
                    rate_limit_id: record.crypto_key_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_rate_limit_lifecycle(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.patientapp.v1.PatientHomeQueryService`.
struct PatientHomeQuerySink {
    client: patientapp::patient_home_query_service_client::PatientHomeQueryServiceClient<Channel>,
}

impl CryptoKeyStatusTarget for PatientHomeQuerySink {
    fn deliver(&self, record: CryptoKeyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_patient_home(patientapp::LookupPatientHomeRequest {
                    patient_home_id: record.crypto_key_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
