//! `relay_fan_sessions_token` reads `acme.fanportal.v1.FanSessionService/ListFanSessions` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::promotions::v1 as promotions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone());
    let found = source
        .list_fan_sessions(fanportal::ListFanSessionsRequest {
            context: Some(context.clone()),
            order_by: "current-relay-fan-sessions-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = found.page_info.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = FanSessionsTokenRecord {
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
        status: result.status,
    };
    dispatch_fan_sessions_token("certificate-authority-workflow", record, channel, &context).await
}

/// What `relay_fan_sessions_token` keeps of each ListFanSessions response.
#[derive(Clone, Debug, Default)]
pub struct FanSessionsTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub status: String,
}

/// Where `relay_fan_sessions_token` hands each record.
trait FanSessionsTokenDestination {
    fn deliver(&self, record: FanSessionsTokenRecord) -> Delivery;
}

/// Writes records to `acme.promotions.v1.PromotionService`.
struct PromotionSink {
    client: promotions::promotion_service_client::PromotionServiceClient<Channel>,
    context: common::RequestContext,
}

impl FanSessionsTokenDestination for PromotionSink {
    fn deliver(&self, record: FanSessionsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(promotions::SearchPromotionsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.next_page_token,
                        ..Default::default()
                    }),
                    order_by: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_promotions(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateAuthorityWorkflowService`.
struct CertificateAuthorityWorkflowSink {
    client:
        certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient<Channel>,
}

impl FanSessionsTokenDestination for CertificateAuthorityWorkflowSink {
    fn deliver(&self, record: FanSessionsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_certificate_authority_workflow(certificates::CancelCertificateAuthorityWorkflowRequest {
                    details: Some(certificates::cancel_certificate_authority_workflow_request::Details {
                        initiated_by: record.next_page_token,
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
struct FanSessionsTokenLogOnly;

impl FanSessionsTokenDestination for FanSessionsTokenLogOnly {
    fn deliver(&self, record: FanSessionsTokenRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.next_page_token);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_fan_sessions_token(
    kind: &str,
    record: FanSessionsTokenRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let promotion = PromotionSink {
        client: promotions::promotion_service_client::PromotionServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let certificate_authority_workflow = CertificateAuthorityWorkflowSink {
        client:
            certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient::new(
                channel,
            ),
    };
    let write: &dyn Fn(FanSessionsTokenRecord) -> Delivery = match kind {
        "promotion" => &|record| promotion.deliver(record),
        "certificate-authority-workflow" => &|record| certificate_authority_workflow.deliver(record),
        _ => &|record| FanSessionsTokenLogOnly.deliver(record),
    };
    write(record).await
}
