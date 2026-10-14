//! `route_throttle_kind` reads `acme.ratelimiting.v1.ThrottleCalculationService/CalculateThrottle` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel.clone());
    let fetched = source
        .calculate_throttle(ratelimiting::CalculateThrottleRequest {
            context: Some(context.clone()),
            throttle_id: "current-route-throttle-kind".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = fetched.throttle.unwrap_or_default();
    let breakdown = fetched.breakdown.unwrap_or_default();
    let record = ThrottleKindRecord {
        kind: throttle.kind,
        correlation_id: throttle.correlation_id,
        currency_code: breakdown.currency_code,
        throttle_id: throttle.throttle_id,
    };
    act_on_throttle_kind("csr-calculation", record, channel, context).await
}

/// What `route_throttle_kind` keeps of each CalculateThrottle response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleKindRecord {
    pub kind: String,
    pub correlation_id: String,
    pub currency_code: String,
    pub throttle_id: String,
}

/// Where `route_throttle_kind` hands each record.
trait ThrottleKindWriter {
    fn write(&self, record: ThrottleKindRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CsrCalculationService`.
struct CsrCalculationSink {
    client: certificates::csr_calculation_service_client::CsrCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleKindWriter for CsrCalculationSink {
    fn write(&self, record: ThrottleKindRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::ExplainCsrCalculationRequest {
                    csr: Some(certificates::Csr {
                        csr_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.explain_csr_calculation(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl ThrottleKindWriter for CryptoKeyCalculationSink {
    fn write(&self, record: ThrottleKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_crypto_key(keymanagement::QuoteCryptoKeyRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.audit.v1.AuditEventService`.
struct AuditEventSink {
    client: audit::audit_event_service_client::AuditEventServiceClient<Channel>,
}

impl ThrottleKindWriter for AuditEventSink {
    fn write(&self, record: ThrottleKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_audit_event(audit::DeleteAuditEventRequest {
                    etag: record.throttle_id,
                    audit_event_id: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
}

impl ThrottleKindWriter for AppointmentSink {
    fn write(&self, record: ThrottleKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_appointments(appointments::ListAppointmentsRequest {
                    filter: Some(appointments::list_appointments_request::Filter {
                        region: record.currency_code,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.kind,
                        ..Default::default()
                    }),
                    order_by: record.throttle_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_throttle_kind(
    kind: &str,
    record: ThrottleKindRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ThrottleKindRecord) -> Delivery>> = BTreeMap::new();
    let csr_calculation_channel = channel.clone();
    let csr_calculation_context = context.clone();
    actions.insert(
        "csr-calculation",
        Box::new(move |record| {
            CsrCalculationSink {
                client: certificates::csr_calculation_service_client::CsrCalculationServiceClient::new(
                    csr_calculation_channel.clone(),
                ),
                context: csr_calculation_context.clone(),
            }
            .write(record)
        }),
    );
    let crypto_key_calculation_channel = channel.clone();
    actions.insert(
        "crypto-key-calculation",
        Box::new(move |record| {
            CryptoKeyCalculationSink {
                client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                    crypto_key_calculation_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let audit_event_channel = channel.clone();
    actions.insert(
        "audit-event",
        Box::new(move |record| {
            AuditEventSink {
                client: audit::audit_event_service_client::AuditEventServiceClient::new(audit_event_channel.clone()),
            }
            .write(record)
        }),
    );
    let appointment_channel = channel.clone();
    actions.insert(
        "appointment",
        Box::new(move |record| {
            AppointmentSink {
                client: appointments::appointment_service_client::AppointmentServiceClient::new(
                    appointment_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
