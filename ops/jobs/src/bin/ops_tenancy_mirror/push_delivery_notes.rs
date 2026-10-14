//! `push_delivery_notes` reads `acme.deliveries.v1.DeliveryQueryService/PeekDelivery` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::fanportal::v1 as fanportal;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone());
    let loaded = source
        .peek_delivery(deliveries::PeekDeliveryRequest {
            context: Some(context.clone()),
            delivery_id: "main-push-delivery-notes".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let delivery = loaded.delivery.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = DeliveryNotesRecord {
        notes: delivery.notes,
        status: result.status,
    };
    act_on_delivery_notes("fan-session-query", record, channel, context).await
}

/// What `push_delivery_notes` keeps of each PeekDelivery response.
#[derive(Clone, Debug, Default)]
pub struct DeliveryNotesRecord {
    pub notes: String,
    pub status: String,
}

/// Where `push_delivery_notes` hands each record.
trait DeliveryNotesTarget {
    fn send(&self, record: DeliveryNotesRecord) -> Delivery;
}

/// Passes records to `acme.fanportal.v1.FanSessionQueryService`.
struct FanSessionQuerySink {
    client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl DeliveryNotesTarget for FanSessionQuerySink {
    fn send(&self, record: DeliveryNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fanportal::PeekFanSessionRequest {
                    fan_session_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_fan_session(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.ApiKeyIntegrationService`.
struct ApiKeyIntegrationSink {
    client: edgegateway::api_key_integration_service_client::ApiKeyIntegrationServiceClient<Channel>,
}

impl DeliveryNotesTarget for ApiKeyIntegrationSink {
    fn send(&self, record: DeliveryNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_api_key_sources(edgegateway::ListApiKeySourcesRequest {
                    filter: Some(edgegateway::list_api_key_sources_request::Filter {
                        owner_id: record.status,
                        region: record.notes,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_delivery_notes(
    kind: &str,
    record: DeliveryNotesRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(DeliveryNotesRecord) -> Delivery>> = BTreeMap::new();
    let fan_session_query_channel = channel.clone();
    let fan_session_query_context = context.clone();
    actions.insert(
        "fan-session-query",
        Box::new(move |record| {
            FanSessionQuerySink {
                client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient::new(
                    fan_session_query_channel.clone(),
                ),
                context: fan_session_query_context.clone(),
            }
            .send(record)
        }),
    );
    let api_key_integration_channel = channel.clone();
    actions.insert(
        "api-key-integration",
        Box::new(move |record| {
            ApiKeyIntegrationSink {
                client: edgegateway::api_key_integration_service_client::ApiKeyIntegrationServiceClient::new(
                    api_key_integration_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
