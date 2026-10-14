//! `mirror_users_id` reads `acme.identity.v1.UserQueryService/FindUsers` and hands what it keeps to
//! `acme.sponsorships.v1.SponsorCalculationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = identity::user_query_service_client::UserQueryServiceClient::new(channel.clone());
    let loaded = source
        .find_users(identity::FindUsersRequest {
            context: Some(context.clone()),
            order_by: "nightly-mirror-users-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = UsersIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
    };
    SponsorCalculationSink {
        client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(channel),
        context: context.clone(),
    }
    .push(record)
    .await
}

/// What `mirror_users_id` keeps of each FindUsers response.
#[derive(Clone, Debug, Default)]
pub struct UsersIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `mirror_users_id` hands each record.
trait UsersIdTarget {
    fn push(&self, record: UsersIdRecord) -> Delivery;
}

/// Sends records to `acme.sponsorships.v1.SponsorCalculationService`.
struct SponsorCalculationSink {
    client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl UsersIdTarget for SponsorCalculationSink {
    fn push(&self, record: UsersIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::RecalculateSponsorsRequest {
                    sponsor: Some(sponsorships::Sponsor {
                        sponsor_id: record.status,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.recalculate_sponsors(request).await?;
            Ok(())
        })
    }
}
