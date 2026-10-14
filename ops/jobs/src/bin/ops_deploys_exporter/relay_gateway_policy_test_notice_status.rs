//! `relay_gateway_policy_test_notice_status` reads
//! `acme.edgegateway.v1.GatewayPolicyNotificationService/SendGatewayPolicyTestNotice` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient::new(
            channel.clone(),
        );
    let found = source
        .send_gateway_policy_test_notice(edgegateway::SendGatewayPolicyTestNoticeRequest {
            context: Some(context.clone()),
            gateway_policy_id: "nightly-relay-gateway-policy-test-notice-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let gateway_policy = found.gateway_policy.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = GatewayPolicyTestNoticeStatusRecord {
        status: outcome.status,
        gateway_policy_id: gateway_policy.gateway_policy_id,
        reference_id: outcome.reference_id,
        result_status: result.status,
    };
    deliver_gateway_policy_test_notice_status("appointment", record, channel, &context).await
}

/// What `relay_gateway_policy_test_notice_status` keeps of each SendGatewayPolicyTestNotice response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyTestNoticeStatusRecord {
    pub status: String,
    pub gateway_policy_id: String,
    pub reference_id: String,
    pub result_status: String,
}

/// Where `relay_gateway_policy_test_notice_status` hands each record.
trait GatewayPolicyTestNoticeStatusTarget {
    fn send(&self, record: GatewayPolicyTestNoticeStatusRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.ThrottleLifecycleService`.
struct ThrottleLifecycleSink {
    client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient<Channel>,
}

impl GatewayPolicyTestNoticeStatusTarget for ThrottleLifecycleSink {
    fn send(&self, record: GatewayPolicyTestNoticeStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_throttle(ratelimiting::RenewThrottleRequest {
                    throttle_id: record.reference_id,
                    details: Some(ratelimiting::renew_throttle_request::Details {
                        reason_code: record.result_status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
}

impl GatewayPolicyTestNoticeStatusTarget for AppointmentSink {
    fn send(&self, record: GatewayPolicyTestNoticeStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_appointments(appointments::ListAppointmentsRequest {
                    filter: Some(appointments::list_appointments_request::Filter {
                        region: record.gateway_policy_id,
                        owner_id: record.result_status,
                        ..Default::default()
                    }),
                    order_by: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct GatewayPolicyTestNoticeStatusLogOnly;

impl GatewayPolicyTestNoticeStatusTarget for GatewayPolicyTestNoticeStatusLogOnly {
    fn send(&self, record: GatewayPolicyTestNoticeStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_gateway_policy_test_notice_status(
    kind: &str,
    record: GatewayPolicyTestNoticeStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn GatewayPolicyTestNoticeStatusTarget> = match kind {
        "throttle-lifecycle" => Box::new(ThrottleLifecycleSink {
            client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
        "appointment" => Box::new(AppointmentSink {
            client: appointments::appointment_service_client::AppointmentServiceClient::new(channel),
        }),
        _ => Box::new(GatewayPolicyTestNoticeStatusLogOnly),
    };
    sink.send(record).await
}
