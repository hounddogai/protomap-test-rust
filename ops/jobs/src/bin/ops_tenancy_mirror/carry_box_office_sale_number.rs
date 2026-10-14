//! `carry_box_office_sale_number` reads `acme.boxoffice.v1.BoxOfficeSaleService/CloneBoxOfficeSale` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient::new(channel.clone());
    let found = source
        .clone_box_office_sale(boxoffice::CloneBoxOfficeSaleRequest {
            context: Some(context.clone()),
            request_id: "default-carry-box-office-sale-number".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let box_office_sale = found.box_office_sale.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = BoxOfficeSaleNumberRecord {
        number: box_office_sale.number,
        status: result.status,
    };
    deliver_box_office_sale_number("feature-flag-sync", record, channel, &context).await
}

/// What `carry_box_office_sale_number` keeps of each CloneBoxOfficeSale response.
#[derive(Clone, Debug, Default)]
pub struct BoxOfficeSaleNumberRecord {
    pub number: String,
    pub status: String,
}

/// Where `carry_box_office_sale_number` hands each record.
trait BoxOfficeSaleNumberDestination {
    fn send(&self, record: BoxOfficeSaleNumberRecord) -> Delivery;
}

/// Passes records to `acme.search.v1.SearchIndexService`.
struct SearchIndexSink {
    client: search::search_index_service_client::SearchIndexServiceClient<Channel>,
}

impl BoxOfficeSaleNumberDestination for SearchIndexSink {
    fn send(&self, record: BoxOfficeSaleNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_search_index(search::GetSearchIndexRequest {
                    search_index_id: record.number,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FeatureFlagSyncService`.
struct FeatureFlagSyncSink {
    client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl BoxOfficeSaleNumberDestination for FeatureFlagSyncSink {
    fn send(&self, record: BoxOfficeSaleNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::PullFeatureFlagsRequest {
                    feature_flag_id: record.status,
                    details: Some(featureflags::pull_feature_flags_request::Details {
                        peer_region: record.number,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.pull_feature_flags(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct BoxOfficeSaleNumberLogOnly;

impl BoxOfficeSaleNumberDestination for BoxOfficeSaleNumberLogOnly {
    fn send(&self, record: BoxOfficeSaleNumberRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.number);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_box_office_sale_number(
    kind: &str,
    record: BoxOfficeSaleNumberRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn BoxOfficeSaleNumberDestination> = match kind {
        "search-index" => Box::new(SearchIndexSink {
            client: search::search_index_service_client::SearchIndexServiceClient::new(channel.clone()),
        }),
        "feature-flag-sync" => Box::new(FeatureFlagSyncSink {
            client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(BoxOfficeSaleNumberLogOnly),
    };
    sink.send(record).await
}
