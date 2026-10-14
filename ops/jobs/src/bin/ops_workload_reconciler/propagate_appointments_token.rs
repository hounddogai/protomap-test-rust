//! `propagate_appointments_token` reads `acme.appointments.v1.AppointmentQueryService/BrowseAppointments` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        appointments::appointment_query_service_client::AppointmentQueryServiceClient::new(channel.clone());
    let loaded = source
        .browse_appointments(appointments::BrowseAppointmentsRequest {
            context: Some(context.clone()),
            order_by: "default-propagate-appointments-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = loaded.page_info.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = AppointmentsTokenRecord {
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
        status: result.status,
        correlation_id: result.correlation_id,
    };
    deliver_appointments_token("config-set-query", record, channel, &context).await
}

/// What `propagate_appointments_token` keeps of each BrowseAppointments response.
#[derive(Clone, Debug, Default)]
pub struct AppointmentsTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `propagate_appointments_token` hands each record.
trait AppointmentsTokenWriter {
    fn write(&self, record: AppointmentsTokenRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.ConfigSetQueryService`.
struct ConfigSetQuerySink {
    client: configuration::config_set_query_service_client::ConfigSetQueryServiceClient<Channel>,
}

impl AppointmentsTokenWriter for ConfigSetQuerySink {
    fn write(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_config_sets(configuration::BrowseConfigSetsRequest {
                    filter: Some(configuration::browse_config_sets_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.seating.v1.SeatMapService`.
struct SeatMapSink {
    client: seating::seat_map_service_client::SeatMapServiceClient<Channel>,
}

impl AppointmentsTokenWriter for SeatMapSink {
    fn write(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_seat_map(seating::DeleteSeatMapRequest {
                    seat_map_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl AppointmentsTokenWriter for RateLimitValidationSink {
    fn write(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_rate_limit_eligibility(ratelimiting::CheckRateLimitEligibilityRequest {
                    rate_limit_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.certificates.v1.CertificateAuthorityCalculationService`.
struct CertificateAuthorityCalculationSink {
    client:
        certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient<
            Channel,
        >,
    context: common::RequestContext,
}

impl AppointmentsTokenWriter for CertificateAuthorityCalculationSink {
    fn write(&self, record: AppointmentsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::GetCertificateAuthorityRatesRequest {
                    certificate_authority_id: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_certificate_authority_rates(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct AppointmentsTokenLogOnly;

impl AppointmentsTokenWriter for AppointmentsTokenLogOnly {
    fn write(&self, record: AppointmentsTokenRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.next_page_token);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_appointments_token(
    kind: &str,
    record: AppointmentsTokenRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AppointmentsTokenWriter> = match kind {
        "config-set-query" => Box::new(ConfigSetQuerySink { client: configuration::config_set_query_service_client::ConfigSetQueryServiceClient::new(channel.clone()) }),
        "seat-map" => Box::new(SeatMapSink { client: seating::seat_map_service_client::SeatMapServiceClient::new(channel.clone()) }),
        "rate-limit-validation" => Box::new(RateLimitValidationSink { client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(channel.clone()) }),
        "certificate-authority-calculation" => Box::new(CertificateAuthorityCalculationSink { client: certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient::new(channel), context: context.clone() }),
        _ => Box::new(AppointmentsTokenLogOnly),
    };
    sink.write(record).await
}
