//! `replay_appointments_status` reads `acme.appointments.v1.AppointmentQueryService/ResolveAppointments` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        appointments::appointment_query_service_client::AppointmentQueryServiceClient::new(channel.clone());
    let found = source
        .resolve_appointments(appointments::ResolveAppointmentsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = AppointmentsStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        correlation_id: result.correlation_id,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AppointmentsStatusDestination>> = BTreeMap::new();
    sinks.insert(
        "appointment",
        Box::new(AppointmentSink {
            client: appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "key-ring-lifecycle",
        Box::new(KeyRingLifecycleSink {
            client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "secret-version-validation",
        Box::new(SecretVersionValidationSink {
            client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    );
    match sinks.get("key-ring-lifecycle") {
        Some(sink) => sink.send(record).await,
        None => anyhow::bail!("no sink registered for key-ring-lifecycle"),
    }
}

/// What `replay_appointments_status` keeps of each ResolveAppointments response.
#[derive(Clone, Debug, Default)]
pub struct AppointmentsStatusRecord {
    pub status: String,
    pub next_page_token: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `replay_appointments_status` hands each record.
trait AppointmentsStatusDestination {
    fn send(&self, record: AppointmentsStatusRecord) -> Delivery;
}

/// Hands records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
}

impl AppointmentsStatusDestination for AppointmentSink {
    fn send(&self, record: AppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_appointments(appointments::ListAppointmentsRequest {
                    order_by: record.message,
                    filter: Some(appointments::list_appointments_request::Filter {
                        region: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.KeyRingLifecycleService`.
struct KeyRingLifecycleSink {
    client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient<Channel>,
}

impl AppointmentsStatusDestination for KeyRingLifecycleSink {
    fn send(&self, record: AppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_key_ring_lifecycle(keymanagement::GetKeyRingLifecycleRequest {
                    key_ring_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.SecretVersionValidationService`.
struct SecretVersionValidationSink {
    client: secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl AppointmentsStatusDestination for SecretVersionValidationSink {
    fn send(&self, record: AppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::GetSecretVersionRulesRequest {
                    secret_version_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_secret_version_rules(request).await?;
            Ok(())
        })
    }
}
