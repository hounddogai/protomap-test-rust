//! `hand_off_feature_flag_status` reads `acme.featureflags.v1.FeatureFlagQueryService/PeekFeatureFlag` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(channel.clone());
    let reply = source
        .peek_feature_flag(featureflags::PeekFeatureFlagRequest {
            context: Some(context.clone()),
            feature_flag_id: "default-hand-off-feature-flag-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = OffFeatureFlagStatusRecord {
        status: result.status,
        message: result.message,
    };
    let sinks: Vec<Box<dyn OffFeatureFlagStatusTarget>> = vec![
        Box::new(FeatureFlagSink {
            client: featureflags::feature_flag_service_client::FeatureFlagServiceClient::new(channel.clone()),
        }),
        Box::new(SponsorQuerySink {
            client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone()),
        }),
        Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[1].write(record).await
}

/// What `hand_off_feature_flag_status` keeps of each PeekFeatureFlag response.
#[derive(Clone, Debug, Default)]
pub struct OffFeatureFlagStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `hand_off_feature_flag_status` hands each record.
trait OffFeatureFlagStatusTarget {
    fn write(&self, record: OffFeatureFlagStatusRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FeatureFlagService`.
struct FeatureFlagSink {
    client: featureflags::feature_flag_service_client::FeatureFlagServiceClient<Channel>,
}

impl OffFeatureFlagStatusTarget for FeatureFlagSink {
    fn write(&self, record: OffFeatureFlagStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_feature_flags(featureflags::ListFeatureFlagsRequest {
                    filter: Some(featureflags::list_feature_flags_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
}

impl OffFeatureFlagStatusTarget for SponsorQuerySink {
    fn write(&self, record: OffFeatureFlagStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_sponsors(sponsorships::SuggestSponsorsRequest {
                    filter: Some(sponsorships::suggest_sponsors_request::Filter {
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffFeatureFlagStatusTarget for AccessGrantValidationSink {
    fn write(&self, record: OffFeatureFlagStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::LintAccessGrantsRequest {
                    access_grant: Some(secrets::AccessGrant {
                        access_grant_id: record.message,
                        ..Default::default()
                    }),
                    details: Some(secrets::lint_access_grants_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_access_grants(request).await?;
            Ok(())
        })
    }
}
