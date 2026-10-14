//! `settle_get_crypto_keys_status` reads `acme.keymanagement.v1.CryptoKeyService/BatchGetCryptoKeys` and
//! hands what it keeps to `acme.admissions.v1.AdmissionService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = keymanagement::crypto_key_service_client::CryptoKeyServiceClient::new(channel.clone());
    let fetched = source
        .batch_get_crypto_keys(keymanagement::BatchGetCryptoKeysRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = GetCryptoKeysStatusRecord {
        status: result.status,
        message: result.message,
    };
    AdmissionSink {
        client: admissions::admission_service_client::AdmissionServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `settle_get_crypto_keys_status` keeps of each BatchGetCryptoKeys response.
#[derive(Clone, Debug, Default)]
pub struct GetCryptoKeysStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `settle_get_crypto_keys_status` hands each record.
trait GetCryptoKeysStatusTarget {
    fn forward(&self, record: GetCryptoKeysStatusRecord) -> Delivery;
}

/// Passes records to `acme.admissions.v1.AdmissionService`.
struct AdmissionSink {
    client: admissions::admission_service_client::AdmissionServiceClient<Channel>,
}

impl GetCryptoKeysStatusTarget for AdmissionSink {
    fn forward(&self, record: GetCryptoKeysStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_admission(admissions::CreateAdmissionRequest {
                    admission: Some(admissions::Admission {
                        correlation_id: record.message,
                        source: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
