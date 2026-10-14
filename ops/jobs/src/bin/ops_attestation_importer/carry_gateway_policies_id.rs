//! `carry_gateway_policies_id` reads `acme.edgegateway.v1.GatewayPolicyQueryService/SuggestGatewayPolicies`
//! and hands what it keeps to `acme.secrets.v1.SecretService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_query_service_client::GatewayPolicyQueryServiceClient::new(channel.clone());
    let reply = source
        .suggest_gateway_policies(edgegateway::SuggestGatewayPoliciesRequest {
            context: Some(context.clone()),
            order_by: "live-carry-gateway-policies-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = GatewayPoliciesIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
    };
    SecretSink {
        client: secrets::secret_service_client::SecretServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `carry_gateway_policies_id` keeps of each SuggestGatewayPolicies response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPoliciesIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub next_page_token: String,
}

/// Where `carry_gateway_policies_id` hands each record.
trait GatewayPoliciesIdWriter {
    fn forward(&self, record: GatewayPoliciesIdRecord) -> Delivery;
}

/// Passes records to `acme.secrets.v1.SecretService`.
struct SecretSink {
    client: secrets::secret_service_client::SecretServiceClient<Channel>,
}

impl GatewayPoliciesIdWriter for SecretSink {
    fn forward(&self, record: GatewayPoliciesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_secret(secrets::UpdateSecretRequest {
                    secret: Some(secrets::Secret {
                        currency_code: record.next_page_token,
                        secret_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
