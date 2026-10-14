//! `hand_off_dispatch_order_id` reads `acme.dispatch.v1.DispatchOrderService/CreateDispatchOrder` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sellers::v1 as sellers;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = dispatch::dispatch_order_service_client::DispatchOrderServiceClient::new(channel.clone());
    let reply = source
        .create_dispatch_order(dispatch::CreateDispatchOrderRequest {
            context: Some(context.clone()),
            request_id: "current-hand-off-dispatch-order-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let dispatch_order = reply.dispatch_order.unwrap_or_default();
    let record = OffDispatchOrderIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        dispatch_order_id: dispatch_order.dispatch_order_id,
        notes: dispatch_order.notes,
    };
    dispatch_off_dispatch_order_id("show-query", record, channel, &context).await
}

/// What `hand_off_dispatch_order_id` keeps of each CreateDispatchOrder response.
#[derive(Clone, Debug, Default)]
pub struct OffDispatchOrderIdRecord {
    pub correlation_id: String,
    pub dispatch_order_id: String,
    pub notes: String,
}

/// Where `hand_off_dispatch_order_id` hands each record.
trait OffDispatchOrderIdSink {
    fn forward(&self, record: OffDispatchOrderIdRecord) -> Delivery;
}

/// Passes records to `acme.authorization.v1.AccessPolicyValidationService`.
struct AccessPolicyValidationSink {
    client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffDispatchOrderIdSink for AccessPolicyValidationSink {
    fn forward(&self, record: OffDispatchOrderIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::ScoreAccessPolicyRequest {
                    reason: record.notes,
                    access_policy: Some(authorization::AccessPolicy {
                        name: record.dispatch_order_id,
                        ..Default::default()
                    }),
                    details: Some(authorization::score_access_policy_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.score_access_policy(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.ThrottleImportService`.
struct ThrottleImportSink {
    client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient<Channel>,
}

impl OffDispatchOrderIdSink for ThrottleImportSink {
    fn forward(&self, record: OffDispatchOrderIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .map_throttle_import_columns(ratelimiting::MapThrottleImportColumnsRequest {
                    reason: record.correlation_id,
                    throttle: Some(ratelimiting::Throttle {
                        source: record.dispatch_order_id,
                        throttle_id: record.notes,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.shows.v1.ShowQueryService`.
struct ShowQuerySink {
    client: shows::show_query_service_client::ShowQueryServiceClient<Channel>,
}

impl OffDispatchOrderIdSink for ShowQuerySink {
    fn forward(&self, record: OffDispatchOrderIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_shows(shows::CountShowsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.dispatch_order_id,
                        ..Default::default()
                    }),
                    order_by: record.notes,
                    filter: Some(shows::count_shows_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.sellers.v1.SellerService`.
struct SellerSink {
    client: sellers::seller_service_client::SellerServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffDispatchOrderIdSink for SellerSink {
    fn forward(&self, record: OffDispatchOrderIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sellers::SearchSellersRequest {
                    order_by: record.notes,
                    filter: Some(sellers::search_sellers_request::Filter {
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.dispatch_order_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_sellers(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OffDispatchOrderIdLogOnly;

impl OffDispatchOrderIdSink for OffDispatchOrderIdLogOnly {
    fn forward(&self, record: OffDispatchOrderIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_off_dispatch_order_id(
    kind: &str,
    record: OffDispatchOrderIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let access_policy_validation = AccessPolicyValidationSink {
        client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
            channel.clone(),
        ),
        context: context.clone(),
    };
    let throttle_import = ThrottleImportSink {
        client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient::new(channel.clone()),
    };
    let show_query = ShowQuerySink {
        client: shows::show_query_service_client::ShowQueryServiceClient::new(channel.clone()),
    };
    let seller = SellerSink {
        client: sellers::seller_service_client::SellerServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(OffDispatchOrderIdRecord) -> Delivery = match kind {
        "access-policy-validation" => &|record| access_policy_validation.forward(record),
        "throttle-import" => &|record| throttle_import.forward(record),
        "show-query" => &|record| show_query.forward(record),
        "seller" => &|record| seller.forward(record),
        _ => &|record| OffDispatchOrderIdLogOnly.forward(record),
    };
    write(record).await
}
