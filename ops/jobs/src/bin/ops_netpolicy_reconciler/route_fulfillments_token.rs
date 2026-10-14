//! `route_fulfillments_token` reads `acme.fulfillment.v1.FulfillmentQueryService/BrowseFulfillments` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient::new(channel.clone());
    let found = source
        .browse_fulfillments(fulfillment::BrowseFulfillmentsRequest {
            context: Some(context.clone()),
            order_by: "default-route-fulfillments-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = found.page_info.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = FulfillmentsTokenRecord {
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn FulfillmentsTokenSink>> = BTreeMap::new();
    sinks.insert(
        "key-ring-reporting",
        Box::new(KeyRingReportingSink {
            client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "access-policy-validation",
        Box::new(AccessPolicyValidationSink {
            client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
                channel,
            ),
        }),
    );
    match sinks.get("key-ring-reporting") {
        Some(sink) => sink.send(record).await,
        None => anyhow::bail!("no sink registered for key-ring-reporting"),
    }
}

/// What `route_fulfillments_token` keeps of each BrowseFulfillments response.
#[derive(Clone, Debug, Default)]
pub struct FulfillmentsTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub status: String,
}

/// Where `route_fulfillments_token` hands each record.
trait FulfillmentsTokenSink {
    fn send(&self, record: FulfillmentsTokenRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRingReportingService`.
struct KeyRingReportingSink {
    client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient<Channel>,
}

impl FulfillmentsTokenSink for KeyRingReportingSink {
    fn send(&self, record: FulfillmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_key_ring_reports(keymanagement::ListKeyRingReportsRequest {
                    filter: Some(keymanagement::list_key_ring_reports_request::Filter {
                        owner_id: record.next_page_token,
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.authorization.v1.AccessPolicyValidationService`.
struct AccessPolicyValidationSink {
    client: authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient<Channel>,
}

impl FulfillmentsTokenSink for AccessPolicyValidationSink {
    fn send(&self, record: FulfillmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_access_policy_eligibility(authorization::CheckAccessPolicyEligibilityRequest {
                    access_policy_id: record.status,
                    details: Some(authorization::check_access_policy_eligibility_request::Details {
                        locale: record.message,
                        ..Default::default()
                    }),
                    access_policy: Some(authorization::AccessPolicy {
                        access_policy_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
