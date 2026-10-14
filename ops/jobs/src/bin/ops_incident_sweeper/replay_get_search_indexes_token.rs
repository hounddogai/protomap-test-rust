//! `replay_get_search_indexes_token` reads `acme.search.v1.SearchIndexService/BatchGetSearchIndexes` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::routing::v1 as routing;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = search::search_index_service_client::SearchIndexServiceClient::new(channel.clone());
    let found = source
        .batch_get_search_indexes(search::BatchGetSearchIndexesRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = found.page_info.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = GetSearchIndexesTokenRecord {
        next_page_token: page_info.next_page_token,
        message: result.message,
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&result.status),
    };
    dispatch_get_search_indexes_token("role-binding-export", record, channel, &context).await
}

/// What `replay_get_search_indexes_token` keeps of each BatchGetSearchIndexes response.
#[derive(Clone, Debug, Default)]
pub struct GetSearchIndexesTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `replay_get_search_indexes_token` hands each record.
trait GetSearchIndexesTokenTarget {
    fn apply(&self, record: GetSearchIndexesTokenRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.RoleBindingExportService`.
struct RoleBindingExportSink {
    client: authorization::role_binding_export_service_client::RoleBindingExportServiceClient<Channel>,
}

impl GetSearchIndexesTokenTarget for RoleBindingExportSink {
    fn apply(&self, record: GetSearchIndexesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_role_binding_exports(authorization::ListRoleBindingExportsRequest {
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.routing.v1.RouteQueryService`.
struct RouteQuerySink {
    client: routing::route_query_service_client::RouteQueryServiceClient<Channel>,
}

impl GetSearchIndexesTokenTarget for RouteQuerySink {
    fn apply(&self, record: GetSearchIndexesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_routes(routing::SummarizeRoutesRequest {
                    details: Some(routing::summarize_routes_request::Details {
                        format: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct GetSearchIndexesTokenLogOnly;

impl GetSearchIndexesTokenTarget for GetSearchIndexesTokenLogOnly {
    fn apply(&self, record: GetSearchIndexesTokenRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.next_page_token);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_get_search_indexes_token(
    kind: &str,
    record: GetSearchIndexesTokenRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let role_binding_export = RoleBindingExportSink {
        client: authorization::role_binding_export_service_client::RoleBindingExportServiceClient::new(channel.clone()),
    };
    let route_query = RouteQuerySink {
        client: routing::route_query_service_client::RouteQueryServiceClient::new(channel),
    };
    let write: &dyn Fn(GetSearchIndexesTokenRecord) -> Delivery = match kind {
        "role-binding-export" => &|record| role_binding_export.apply(record),
        "route-query" => &|record| route_query.apply(record),
        _ => &|record| GetSearchIndexesTokenLogOnly.apply(record),
    };
    write(record).await
}
