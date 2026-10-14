//! `export_forum_id` reads `acme.community.v1.ForumService/UpdateForum` and hands what it keeps to
//! `acme.personalization.v1.ProfileCalculationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::personalization::v1 as personalization;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = community::forum_service_client::ForumServiceClient::new(channel.clone());
    let loaded = source
        .update_forum(community::UpdateForumRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let forum = loaded.forum.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = ForumIdRecord {
        parent_id: forum.parent_id,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    ProfileCalculationSink {
        client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient::new(channel),
        context: context.clone(),
    }
    .apply(record)
    .await
}

/// What `export_forum_id` keeps of each UpdateForum response.
#[derive(Clone, Debug, Default)]
pub struct ForumIdRecord {
    pub parent_id: String,
    pub correlation_id: String,
}

/// Where `export_forum_id` hands each record.
trait ForumIdSink {
    fn apply(&self, record: ForumIdRecord) -> Delivery;
}

/// Sends records to `acme.personalization.v1.ProfileCalculationService`.
struct ProfileCalculationSink {
    client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ForumIdSink for ProfileCalculationSink {
    fn apply(&self, record: ForumIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(personalization::ExplainProfileCalculationRequest {
                    profile_id: record.correlation_id,
                    reason: record.parent_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.explain_profile_calculation(request).await?;
            Ok(())
        })
    }
}
