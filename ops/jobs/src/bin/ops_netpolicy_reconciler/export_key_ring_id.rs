//! `export_key_ring_id` reads `acme.keymanagement.v1.KeyRingValidationService/ScoreKeyRing` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient::new(channel.clone());
    let fetched = source
        .score_key_ring(keymanagement::ScoreKeyRingRequest {
            context: Some(context.clone()),
            key_ring_id: "default-export-key-ring-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let key_ring = fetched.key_ring.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let breakdown = fetched.breakdown.unwrap_or_default();
    let record = KeyRingIdRecord {
        parent_id: key_ring.parent_id,
        status: result.status,
        currency_code: breakdown.currency_code,
    };
    let mut sinks: BTreeMap<String, Box<dyn KeyRingIdDestination>> = BTreeMap::new();
    sinks.insert(
        "certificate-authority-query".to_owned(),
        Box::new(CertificateAuthorityQuerySink {
            client:
                certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient::new(
                    channel.clone(),
                ),
        }),
    );
    sinks.insert(
        "access-grant-workflow".to_owned(),
        Box::new(AccessGrantWorkflowSink {
            client: secrets::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient::new(channel),
        }),
    );
    let target = std::env::var("EXPORT_KEY_RING_ID_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.push(record).await,
        None => Ok(()),
    }
}

/// What `export_key_ring_id` keeps of each ScoreKeyRing response.
#[derive(Clone, Debug, Default)]
pub struct KeyRingIdRecord {
    pub parent_id: String,
    pub status: String,
    pub currency_code: String,
}

/// Where `export_key_ring_id` hands each record.
trait KeyRingIdDestination {
    fn push(&self, record: KeyRingIdRecord) -> Delivery;
}

/// Writes records to `acme.certificates.v1.CertificateAuthorityQueryService`.
struct CertificateAuthorityQuerySink {
    client: certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient<Channel>,
}

impl KeyRingIdDestination for CertificateAuthorityQuerySink {
    fn push(&self, record: KeyRingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_certificate_authorities(certificates::FindCertificateAuthoritiesRequest {
                    filter: Some(certificates::find_certificate_authorities_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    order_by: record.parent_id,
                    page: Some(common::PageRequest {
                        page_token: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.secrets.v1.AccessGrantWorkflowService`.
struct AccessGrantWorkflowSink {
    client: secrets::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient<Channel>,
}

impl KeyRingIdDestination for AccessGrantWorkflowSink {
    fn push(&self, record: KeyRingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_access_grant_workflow(secrets::AdvanceAccessGrantWorkflowRequest {
                    reason: record.parent_id,
                    details: Some(secrets::advance_access_grant_workflow_request::Details {
                        step: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
