//! `backfill_api_key_id` reads `acme.edgegateway.v1.ApiKeyQueryService/PeekApiKey` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::api_key_query_service_client::ApiKeyQueryServiceClient::new(channel.clone());
    let loaded = source
        .peek_api_key(edgegateway::PeekApiKeyRequest {
            context: Some(context.clone()),
            api_key_id: "default-backfill-api-key-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let api_key = loaded.api_key.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = ApiKeyIdRecord {
        api_key_id: api_key.api_key_id,
        message: result.message,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn ApiKeyIdWriter>> = vec![
        Box::new(SellerSink {
            client: sellers::seller_service_client::SellerServiceClient::new(channel.clone()),
        }),
        Box::new(AdmissionQuerySink {
            client: admissions::admission_query_service_client::AdmissionQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(AccessPolicyQuerySink {
            client: authorization::access_policy_query_service_client::AccessPolicyQueryServiceClient::new(channel),
        }),
    ];
    sinks[0].send(record).await
}

/// What `backfill_api_key_id` keeps of each PeekApiKey response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeyIdRecord {
    pub api_key_id: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `backfill_api_key_id` hands each record.
trait ApiKeyIdWriter {
    fn send(&self, record: ApiKeyIdRecord) -> Delivery;
}

/// Sends records to `acme.sellers.v1.SellerService`.
struct SellerSink {
    client: sellers::seller_service_client::SellerServiceClient<Channel>,
}

impl ApiKeyIdWriter for SellerSink {
    fn send(&self, record: ApiKeyIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_seller(sellers::CreateSellerRequest {
                    request_id: record.message,
                    seller: Some(sellers::Seller {
                        seller_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.admissions.v1.AdmissionQueryService`.
struct AdmissionQuerySink {
    client: admissions::admission_query_service_client::AdmissionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeyIdWriter for AdmissionQuerySink {
    fn send(&self, record: ApiKeyIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(admissions::CountAdmissionsRequest {
                    filter: Some(admissions::count_admissions_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_admissions(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.AccessPolicyQueryService`.
struct AccessPolicyQuerySink {
    client: authorization::access_policy_query_service_client::AccessPolicyQueryServiceClient<Channel>,
}

impl ApiKeyIdWriter for AccessPolicyQuerySink {
    fn send(&self, record: ApiKeyIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_access_policies(authorization::FindAccessPoliciesRequest {
                    filter: Some(authorization::find_access_policies_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
