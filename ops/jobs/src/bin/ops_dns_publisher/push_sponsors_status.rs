//! `push_sponsors_status` reads `acme.sponsorships.v1.SponsorQueryService/FindSponsors` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone());
    let reply = source
        .find_sponsors(sponsorships::FindSponsorsRequest {
            context: Some(context.clone()),
            order_by: "main-push-sponsors-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = SponsorsStatusRecord {
        status: result.status,
        message: result.message,
        next_page_token: page_info.next_page_token,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn SponsorsStatusDestination>> = vec![
        Box::new(PermissionCheckNotificationSink { client: authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(channel.clone()) }),
        Box::new(ConfigChangeApprovalSink { client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(channel), context: context.clone() }),
    ];
    sinks[0].apply(record).await
}

/// What `push_sponsors_status` keeps of each FindSponsors response.
#[derive(Clone, Debug, Default)]
pub struct SponsorsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `push_sponsors_status` hands each record.
trait SponsorsStatusDestination {
    fn apply(&self, record: SponsorsStatusRecord) -> Delivery;
}

/// Writes records to `acme.authorization.v1.PermissionCheckNotificationService`.
struct PermissionCheckNotificationSink {
    client:
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient<Channel>,
}

impl SponsorsStatusDestination for PermissionCheckNotificationSink {
    fn apply(&self, record: SponsorsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .send_permission_check_test_notice(authorization::SendPermissionCheckTestNoticeRequest {
                    permission_check_id: record.message,
                    details: Some(authorization::send_permission_check_test_notice_request::Details {
                        channel: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.ConfigChangeApprovalService`.
struct ConfigChangeApprovalSink {
    client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl SponsorsStatusDestination for ConfigChangeApprovalSink {
    fn apply(&self, record: SponsorsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::RejectConfigChangeRequest {
                    config_change_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.reject_config_change(request).await?;
            Ok(())
        })
    }
}
