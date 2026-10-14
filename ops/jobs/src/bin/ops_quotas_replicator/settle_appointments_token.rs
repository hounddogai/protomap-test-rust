//! `settle_appointments_token` reads `acme.appointments.v1.AppointmentQueryService/SuggestAppointments` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        appointments::appointment_query_service_client::AppointmentQueryServiceClient::new(channel.clone());
    let fetched = source
        .suggest_appointments(appointments::SuggestAppointmentsRequest {
            context: Some(context.clone()),
            order_by: "default-settle-appointments-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = fetched.page_info.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = AppointmentsTokenRecord {
        next_page_token: page_info.next_page_token,
        status: acme_kit::text::normalize(&result.status),
        correlation_id: result.correlation_id,
    };
    dispatch_appointments_token("box-office-sale", record, channel, &context).await
}

/// What `settle_appointments_token` keeps of each SuggestAppointments response.
#[derive(Clone, Debug, Default)]
pub struct AppointmentsTokenRecord {
    pub next_page_token: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `settle_appointments_token` hands each record.
trait AppointmentsTokenTarget {
    fn deliver(&self, record: AppointmentsTokenRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyLifecycleService`.
struct CryptoKeyLifecycleSink {
    client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient<Channel>,
}

impl AppointmentsTokenTarget for CryptoKeyLifecycleSink {
    fn deliver(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_crypto_key(keymanagement::RenewCryptoKeyRequest {
                    crypto_key_id: record.correlation_id,
                    details: Some(keymanagement::renew_crypto_key_request::Details {
                        reason_code: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.audit.v1.AuditEventValidationService`.
struct AuditEventValidationSink {
    client: audit::audit_event_validation_service_client::AuditEventValidationServiceClient<Channel>,
}

impl AppointmentsTokenTarget for AuditEventValidationSink {
    fn deliver(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_audit_event(audit::ScoreAuditEventRequest {
                    audit_event_id: record.next_page_token,
                    audit_event: Some(audit::AuditEvent {
                        source: record.correlation_id,
                        audit_event_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.boxoffice.v1.BoxOfficeSaleService`.
struct BoxOfficeSaleSink {
    client: boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient<Channel>,
}

impl AppointmentsTokenTarget for BoxOfficeSaleSink {
    fn deliver(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_box_office_sales(boxoffice::SearchBoxOfficeSalesRequest {
                    filter: Some(boxoffice::search_box_office_sales_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigSetCalculationService`.
struct ConfigSetCalculationSink {
    client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient<Channel>,
}

impl AppointmentsTokenTarget for ConfigSetCalculationSink {
    fn deliver(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_config_set_options(configuration::CompareConfigSetOptionsRequest {
                    config_set: Some(configuration::ConfigSet {
                        config_set_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct AppointmentsTokenLogOnly;

impl AppointmentsTokenTarget for AppointmentsTokenLogOnly {
    fn deliver(&self, record: AppointmentsTokenRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.next_page_token);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_appointments_token(
    kind: &str,
    record: AppointmentsTokenRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let crypto_key_lifecycle = CryptoKeyLifecycleSink {
        client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(
            channel.clone(),
        ),
    };
    let audit_event_validation = AuditEventValidationSink {
        client: audit::audit_event_validation_service_client::AuditEventValidationServiceClient::new(channel.clone()),
    };
    let box_office_sale = BoxOfficeSaleSink {
        client: boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient::new(channel.clone()),
    };
    let config_set_calculation = ConfigSetCalculationSink {
        client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(channel),
    };
    let write: &dyn Fn(AppointmentsTokenRecord) -> Delivery = match kind {
        "crypto-key-lifecycle" => &|record| crypto_key_lifecycle.deliver(record),
        "audit-event-validation" => &|record| audit_event_validation.deliver(record),
        "box-office-sale" => &|record| box_office_sale.deliver(record),
        "config-set-calculation" => &|record| config_set_calculation.deliver(record),
        _ => &|record| AppointmentsTokenLogOnly.deliver(record),
    };
    write(record).await
}
