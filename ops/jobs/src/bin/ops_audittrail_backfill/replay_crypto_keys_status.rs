//! `replay_crypto_keys_status` reads `acme.keymanagement.v1.CryptoKeyQueryService/FindCryptoKeys` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(channel.clone());
    let reply = source
        .find_crypto_keys(keymanagement::FindCryptoKeysRequest {
            context: Some(context.clone()),
            order_by: "primary-replay-crypto-keys-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = CryptoKeysStatusRecord {
        status: result.status,
        next_page_token: page_info.next_page_token,
        correlation_id: result.correlation_id,
        message: result.message,
    };
    deliver_crypto_keys_status("permission-check-notification", record, channel, &context).await
}

/// What `replay_crypto_keys_status` keeps of each FindCryptoKeys response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeysStatusRecord {
    pub status: String,
    pub next_page_token: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `replay_crypto_keys_status` hands each record.
trait CryptoKeysStatusWriter {
    fn send(&self, record: CryptoKeysStatusRecord) -> Delivery;
}

/// Passes records to `acme.venues.v1.VenueValidationService`.
struct VenueValidationSink {
    client: venues::venue_validation_service_client::VenueValidationServiceClient<Channel>,
}

impl CryptoKeysStatusWriter for VenueValidationSink {
    fn send(&self, record: CryptoKeysStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_venue_eligibility(venues::CheckVenueEligibilityRequest {
                    venue_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.authorization.v1.PermissionCheckNotificationService`.
struct PermissionCheckNotificationSink {
    client:
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeysStatusWriter for PermissionCheckNotificationSink {
    fn send(&self, record: CryptoKeysStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::ListPermissionCheckSubscriptionsRequest {
                    filter: Some(authorization::list_permission_check_subscriptions_request::Filter {
                        region: record.next_page_token,
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_permission_check_subscriptions(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
}

impl CryptoKeysStatusWriter for ExperimentCalculationSink {
    fn send(&self, record: CryptoKeysStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_experiment(experiments::QuoteExperimentRequest {
                    details: Some(experiments::quote_experiment_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    experiment: Some(experiments::Experiment {
                        experiment_id: record.next_page_token,
                        ..Default::default()
                    }),
                    experiment_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct CryptoKeysStatusLogOnly;

impl CryptoKeysStatusWriter for CryptoKeysStatusLogOnly {
    fn send(&self, record: CryptoKeysStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_crypto_keys_status(
    kind: &str,
    record: CryptoKeysStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn CryptoKeysStatusWriter> = match kind {
        "venue-validation" => Box::new(VenueValidationSink { client: venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone()) }),
        "permission-check-notification" => Box::new(PermissionCheckNotificationSink { client: authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(channel.clone()), context: context.clone() }),
        "experiment-calculation" => Box::new(ExperimentCalculationSink { client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(channel) }),
        _ => Box::new(CryptoKeysStatusLogOnly),
    };
    sink.send(record).await
}
