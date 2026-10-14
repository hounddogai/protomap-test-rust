//! `route_api_keys_token` reads `acme.edgegateway.v1.ApiKeyService/ListApiKeys` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::api_key_service_client::ApiKeyServiceClient::new(channel.clone());
    let fetched = source
        .list_api_keys(edgegateway::ListApiKeysRequest {
            context: Some(context.clone()),
            order_by: "main-route-api-keys-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = fetched.page_info.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = ApiKeysTokenRecord {
        next_page_token: page_info.next_page_token,
        status: result.status,
        correlation_id: result.correlation_id,
        message: result.message,
    };
    let sinks: Vec<Box<dyn ApiKeysTokenSink>> = vec![
        Box::new(SeatMapCalculationSink {
            client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient::new(channel.clone()),
        }),
        Box::new(QuotaBucketValidationSink {
            client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(SearchIndexSink {
            client: search::search_index_service_client::SearchIndexServiceClient::new(channel),
        }),
    ];
    sinks[1].push(record).await
}

/// What `route_api_keys_token` keeps of each ListApiKeys response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeysTokenRecord {
    pub next_page_token: String,
    pub status: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `route_api_keys_token` hands each record.
trait ApiKeysTokenSink {
    fn push(&self, record: ApiKeysTokenRecord) -> Delivery;
}

/// Writes records to `acme.seating.v1.SeatMapCalculationService`.
struct SeatMapCalculationSink {
    client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient<Channel>,
}

impl ApiKeysTokenSink for SeatMapCalculationSink {
    fn push(&self, record: ApiKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_seat_map_calculation(seating::ExplainSeatMapCalculationRequest {
                    seat_map: Some(seating::SeatMap {
                        parent_id: record.message,
                        seat_map_id: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(seating::explain_seat_map_calculation_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
}

impl ApiKeysTokenSink for QuotaBucketValidationSink {
    fn push(&self, record: ApiKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_quota_buckets(ratelimiting::LintQuotaBucketsRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.search.v1.SearchIndexService`.
struct SearchIndexSink {
    client: search::search_index_service_client::SearchIndexServiceClient<Channel>,
}

impl ApiKeysTokenSink for SearchIndexSink {
    fn push(&self, record: ApiKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_search_index(search::DeleteSearchIndexRequest {
                    reason: record.next_page_token,
                    etag: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
