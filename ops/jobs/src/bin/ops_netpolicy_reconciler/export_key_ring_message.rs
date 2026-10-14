//! `export_key_ring_message` reads `acme.keymanagement.v1.KeyRingApprovalService/RecallKeyRing` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::forms::v1 as forms;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel.clone());
    let loaded = source
        .recall_key_ring(keymanagement::RecallKeyRingRequest {
            context: Some(context.clone()),
            key_ring_id: "live-export-key-ring-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = KeyRingMessageRecord {
        message: result.message,
        reference_id: outcome.reference_id,
    };
    deliver_key_ring_message("config-set-reporting", record, channel, &context).await
}

/// What `export_key_ring_message` keeps of each RecallKeyRing response.
#[derive(Clone, Debug, Default)]
pub struct KeyRingMessageRecord {
    pub message: String,
    pub reference_id: String,
}

/// Where `export_key_ring_message` hands each record.
trait KeyRingMessageTarget {
    fn deliver(&self, record: KeyRingMessageRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.ConfigSetReportingService`.
struct ConfigSetReportingSink {
    client: configuration::config_set_reporting_service_client::ConfigSetReportingServiceClient<Channel>,
}

impl KeyRingMessageTarget for ConfigSetReportingSink {
    fn deliver(&self, record: KeyRingMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .share_config_set_report(configuration::ShareConfigSetReportRequest {
                    details: Some(configuration::share_config_set_report_request::Details {
                        template_id: record.message,
                        ..Default::default()
                    }),
                    config_set_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.RouteBindingGatewayService`.
struct RouteBindingGatewaySink {
    client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient<Channel>,
    context: common::RequestContext,
}

impl KeyRingMessageTarget for RouteBindingGatewaySink {
    fn deliver(&self, record: KeyRingMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::TrackRouteBindingRequest {
                    details: Some(edgegateway::track_route_binding_request::Details {
                        locale: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.track_route_binding(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.forms.v1.FormCalculationService`.
struct FormCalculationSink {
    client: forms::form_calculation_service_client::FormCalculationServiceClient<Channel>,
}

impl KeyRingMessageTarget for FormCalculationSink {
    fn deliver(&self, record: KeyRingMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_form(forms::EstimateFormRequest {
                    form: Some(forms::Form {
                        number: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleSyncService`.
struct ThrottleSyncSink {
    client: ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient<Channel>,
}

impl KeyRingMessageTarget for ThrottleSyncSink {
    fn deliver(&self, record: KeyRingMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_throttle_sync_state(ratelimiting::GetThrottleSyncStateRequest {
                    throttle_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct KeyRingMessageLogOnly;

impl KeyRingMessageTarget for KeyRingMessageLogOnly {
    fn deliver(&self, record: KeyRingMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_key_ring_message(
    kind: &str,
    record: KeyRingMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn KeyRingMessageTarget> = match kind {
        "config-set-reporting" => Box::new(ConfigSetReportingSink {
            client: configuration::config_set_reporting_service_client::ConfigSetReportingServiceClient::new(
                channel.clone(),
            ),
        }),
        "route-binding-gateway" => Box::new(RouteBindingGatewaySink {
            client: edgegateway::route_binding_gateway_service_client::RouteBindingGatewayServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "form-calculation" => Box::new(FormCalculationSink {
            client: forms::form_calculation_service_client::FormCalculationServiceClient::new(channel.clone()),
        }),
        "throttle-sync" => Box::new(ThrottleSyncSink {
            client: ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient::new(channel),
        }),
        _ => Box::new(KeyRingMessageLogOnly),
    };
    sink.deliver(record).await
}
