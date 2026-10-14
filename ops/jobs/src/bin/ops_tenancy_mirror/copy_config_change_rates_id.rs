//! `copy_config_change_rates_id` reads
//! `acme.configuration.v1.ConfigChangeCalculationService/GetConfigChangeRates` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient::new(
        channel.clone(),
    );
    let reply = source
        .get_config_change_rates(configuration::GetConfigChangeRatesRequest {
            context: Some(context.clone()),
            config_change_id: "current-copy-config-change-rates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let config_change = reply.config_change.unwrap_or_default();
    let record = ConfigChangeRatesIdRecord {
        correlation_id: result.correlation_id,
        kind: config_change.kind,
    };
    let sinks: Vec<Box<dyn ConfigChangeRatesIdWriter>> = vec![
        Box::new(VehicleSink {
            client: fleet::vehicle_service_client::VehicleServiceClient::new(channel.clone()),
        }),
        Box::new(FeatureFlagQuerySink {
            client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(channel),
        }),
    ];
    sinks[1].write(record).await
}

/// What `copy_config_change_rates_id` keeps of each GetConfigChangeRates response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeRatesIdRecord {
    pub correlation_id: String,
    pub kind: String,
}

/// Where `copy_config_change_rates_id` hands each record.
trait ConfigChangeRatesIdWriter {
    fn write(&self, record: ConfigChangeRatesIdRecord) -> Delivery;
}

/// Passes records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
}

impl ConfigChangeRatesIdWriter for VehicleSink {
    fn write(&self, record: ConfigChangeRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_vehicle(fleet::DeleteVehicleRequest {
                    etag: record.correlation_id,
                    reason: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FeatureFlagQueryService`.
struct FeatureFlagQuerySink {
    client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient<Channel>,
}

impl ConfigChangeRatesIdWriter for FeatureFlagQuerySink {
    fn write(&self, record: ConfigChangeRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_feature_flags(featureflags::BrowseFeatureFlagsRequest {
                    filter: Some(featureflags::browse_feature_flags_request::Filter {
                        region: record.kind,
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
}

impl ConfigChangeRatesIdWriter for AccessGrantValidationSink {
    fn write(&self, record: ConfigChangeRatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_access_grant(secrets::NormalizeAccessGrantRequest {
                    access_grant: Some(secrets::AccessGrant {
                        name: record.correlation_id,
                        access_grant_id: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
