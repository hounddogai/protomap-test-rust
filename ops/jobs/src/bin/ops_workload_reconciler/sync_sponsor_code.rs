//! `sync_sponsor_code` reads `acme.sponsorships.v1.SponsorCalculationService/CalculateSponsor` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::forms::v1 as forms;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(channel.clone());
    let found = source
        .calculate_sponsor(sponsorships::CalculateSponsorRequest {
            context: Some(context.clone()),
            sponsor_id: "default-sync-sponsor-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let sponsor = found.sponsor.unwrap_or_default();
    let record = SponsorCodeRecord {
        currency_code: breakdown.currency_code,
        message: result.message,
        sponsor_id: acme_kit::text::normalize(&sponsor.sponsor_id),
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn SponsorCodeWriter>> = vec![
        Box::new(VenueValidationSink {
            client: venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(FormCalculationSink {
            client: forms::form_calculation_service_client::FormCalculationServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(PermissionCheckExportSink {
            client: authorization::permission_check_export_service_client::PermissionCheckExportServiceClient::new(
                channel,
            ),
        }),
    ];
    sinks[2].forward(record).await
}

/// What `sync_sponsor_code` keeps of each CalculateSponsor response.
#[derive(Clone, Debug, Default)]
pub struct SponsorCodeRecord {
    pub currency_code: String,
    pub message: String,
    pub sponsor_id: String,
    pub correlation_id: String,
}

/// Where `sync_sponsor_code` hands each record.
trait SponsorCodeWriter {
    fn forward(&self, record: SponsorCodeRecord) -> Delivery;
}

/// Passes records to `acme.venues.v1.VenueValidationService`.
struct VenueValidationSink {
    client: venues::venue_validation_service_client::VenueValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SponsorCodeWriter for VenueValidationSink {
    fn forward(&self, record: SponsorCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(venues::ValidateVenueRequest {
                    venue: Some(venues::Venue {
                        code: record.message,
                        venue_id: record.sponsor_id,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.validate_venue(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.forms.v1.FormCalculationService`.
struct FormCalculationSink {
    client: forms::form_calculation_service_client::FormCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SponsorCodeWriter for FormCalculationSink {
    fn forward(&self, record: SponsorCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(forms::GetFormRatesRequest {
                    form_id: record.sponsor_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_form_rates(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.PermissionCheckExportService`.
struct PermissionCheckExportSink {
    client: authorization::permission_check_export_service_client::PermissionCheckExportServiceClient<Channel>,
}

impl SponsorCodeWriter for PermissionCheckExportSink {
    fn forward(&self, record: SponsorCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_permission_check_export(authorization::CancelPermissionCheckExportRequest {
                    reason: record.message,
                    permission_check_id: record.sponsor_id,
                    details: Some(authorization::cancel_permission_check_export_request::Details {
                        reason_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
