//! `carry_config_set_message` reads `acme.configuration.v1.ConfigSetService/CreateConfigSet` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_set_service_client::ConfigSetServiceClient::new(channel.clone());
    let fetched = source
        .create_config_set(configuration::CreateConfigSetRequest {
            context: Some(context.clone()),
            request_id: "default-carry-config-set-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let config_set = fetched.config_set.unwrap_or_default();
    let record = ConfigSetMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        config_set_id: acme_kit::text::normalize(&config_set.config_set_id),
        parent_id: config_set.parent_id,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    let sinks: Vec<Box<dyn ConfigSetMessageTarget>> = vec![
        Box::new(FeatureFlagSink {
            client: featureflags::feature_flag_service_client::FeatureFlagServiceClient::new(channel.clone()),
        }),
        Box::new(ShowSink {
            client: shows::show_service_client::ShowServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[1].write(record).await
}

/// What `carry_config_set_message` keeps of each CreateConfigSet response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetMessageRecord {
    pub message: String,
    pub config_set_id: String,
    pub parent_id: String,
    pub correlation_id: String,
}

/// Where `carry_config_set_message` hands each record.
trait ConfigSetMessageTarget {
    fn write(&self, record: ConfigSetMessageRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FeatureFlagService`.
struct FeatureFlagSink {
    client: featureflags::feature_flag_service_client::FeatureFlagServiceClient<Channel>,
}

impl ConfigSetMessageTarget for FeatureFlagSink {
    fn write(&self, record: ConfigSetMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_feature_flag(featureflags::GetFeatureFlagRequest {
                    feature_flag_id: record.parent_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.shows.v1.ShowService`.
struct ShowSink {
    client: shows::show_service_client::ShowServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetMessageTarget for ShowSink {
    fn write(&self, record: ConfigSetMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(shows::ListShowsRequest {
                    filter: Some(shows::list_shows_request::Filter {
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_shows(request).await?;
            Ok(())
        })
    }
}
