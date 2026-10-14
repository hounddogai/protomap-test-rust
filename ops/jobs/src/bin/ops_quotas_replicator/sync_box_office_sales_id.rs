//! `sync_box_office_sales_id` reads `acme.boxoffice.v1.BoxOfficeSaleQueryService/FindBoxOfficeSales` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(channel.clone());
    let loaded = source
        .find_box_office_sales(boxoffice::FindBoxOfficeSalesRequest {
            context: Some(context.clone()),
            order_by: "nightly-sync-box-office-sales-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = BoxOfficeSalesIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn BoxOfficeSalesIdDestination>)> = vec![
        (
            "fulfillment-query",
            Box::new(FulfillmentQuerySink {
                client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "flag-evaluation-export",
            Box::new(FlagEvaluationExportSink {
                client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient::new(
                    channel,
                ),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "flag-evaluation-export" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `sync_box_office_sales_id` keeps of each FindBoxOfficeSales response.
#[derive(Clone, Debug, Default)]
pub struct BoxOfficeSalesIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
    pub status: String,
}

/// Where `sync_box_office_sales_id` hands each record.
trait BoxOfficeSalesIdDestination {
    fn deliver(&self, record: BoxOfficeSalesIdRecord) -> Delivery;
}

/// Passes records to `acme.fulfillment.v1.FulfillmentQueryService`.
struct FulfillmentQuerySink {
    client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl BoxOfficeSalesIdDestination for FulfillmentQuerySink {
    fn deliver(&self, record: BoxOfficeSalesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fulfillment::CountFulfillmentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.next_page_token,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_fulfillments(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagEvaluationExportService`.
struct FlagEvaluationExportSink {
    client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient<Channel>,
    context: common::RequestContext,
}

impl BoxOfficeSalesIdDestination for FlagEvaluationExportSink {
    fn deliver(&self, record: BoxOfficeSalesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::RetryFlagEvaluationExportRequest {
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        flag_evaluation_id: record.correlation_id,
                        correlation_id: record.next_page_token,
                        ..Default::default()
                    }),
                    details: Some(featureflags::retry_flag_evaluation_export_request::Details {
                        step: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.retry_flag_evaluation_export(request).await?;
            Ok(())
        })
    }
}
