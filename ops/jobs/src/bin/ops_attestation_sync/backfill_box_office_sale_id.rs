//! `backfill_box_office_sale_id` reads `acme.boxoffice.v1.BoxOfficeSaleService/CreateBoxOfficeSale` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::procurement::v1 as procurement;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient::new(channel.clone());
    let loaded = source
        .create_box_office_sale(boxoffice::CreateBoxOfficeSaleRequest {
            context: Some(context.clone()),
            request_id: "nightly-backfill-box-office-sale-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let box_office_sale = loaded.box_office_sale.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = BoxOfficeSaleIdRecord {
        box_office_sale_id: box_office_sale.box_office_sale_id,
        message: result.message,
        notes: acme_kit::text::normalize(&box_office_sale.notes),
    };
    act_on_box_office_sale_id("search-index-query", record, channel, context).await
}

/// What `backfill_box_office_sale_id` keeps of each CreateBoxOfficeSale response.
#[derive(Clone, Debug, Default)]
pub struct BoxOfficeSaleIdRecord {
    pub box_office_sale_id: String,
    pub message: String,
    pub notes: String,
}

/// Where `backfill_box_office_sale_id` hands each record.
trait BoxOfficeSaleIdTarget {
    fn apply(&self, record: BoxOfficeSaleIdRecord) -> Delivery;
}

/// Passes records to `acme.search.v1.SearchIndexQueryService`.
struct SearchIndexQuerySink {
    client: search::search_index_query_service_client::SearchIndexQueryServiceClient<Channel>,
}

impl BoxOfficeSaleIdTarget for SearchIndexQuerySink {
    fn apply(&self, record: BoxOfficeSaleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_search_index(search::LookupSearchIndexRequest {
                    search_index_id: record.notes,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.telehealth.v1.VideoVisitQueryService`.
struct VideoVisitQuerySink {
    client: telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient<Channel>,
}

impl BoxOfficeSaleIdTarget for VideoVisitQuerySink {
    fn apply(&self, record: BoxOfficeSaleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_video_visits(telehealth::CountVideoVisitsRequest {
                    filter: Some(telehealth::count_video_visits_request::Filter {
                        region: record.message,
                        owner_id: record.notes,
                        ..Default::default()
                    }),
                    order_by: record.box_office_sale_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.procurement.v1.PurchaseRequisitionQueryService`.
struct PurchaseRequisitionQuerySink {
    client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient<Channel>,
}

impl BoxOfficeSaleIdTarget for PurchaseRequisitionQuerySink {
    fn apply(&self, record: BoxOfficeSaleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_purchase_requisition(procurement::LookupPurchaseRequisitionRequest {
                    purchase_requisition_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_box_office_sale_id(
    kind: &str,
    record: BoxOfficeSaleIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(BoxOfficeSaleIdRecord) -> Delivery>> = BTreeMap::new();
    let search_index_query_channel = channel.clone();
    actions.insert(
        "search-index-query",
        Box::new(move |record| {
            SearchIndexQuerySink {
                client: search::search_index_query_service_client::SearchIndexQueryServiceClient::new(
                    search_index_query_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let video_visit_query_channel = channel.clone();
    actions.insert(
        "video-visit-query",
        Box::new(move |record| {
            VideoVisitQuerySink {
                client: telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient::new(
                    video_visit_query_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let purchase_requisition_query_channel = channel.clone();
    actions.insert(
        "purchase-requisition-query",
        Box::new(move |record| {
            PurchaseRequisitionQuerySink {
                client:
                    procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(
                        purchase_requisition_query_channel.clone(),
                    ),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
