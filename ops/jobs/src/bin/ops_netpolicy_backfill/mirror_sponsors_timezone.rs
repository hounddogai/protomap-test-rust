//! `mirror_sponsors_timezone` reads `acme.sponsorships.v1.SponsorQueryService/SummarizeSponsors` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::onboarding::v1 as onboarding;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone());
    let found = source
        .summarize_sponsors(sponsorships::SummarizeSponsorsRequest {
            context: Some(context.clone()),
            reason: "default-mirror-sponsors-timezone".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let period = found.period.unwrap_or_default();
    let record = SponsorsTimezoneRecord {
        timezone: period.timezone,
        report_id: found.report_id,
    };
    let sinks: Vec<(&str, Box<dyn SponsorsTimezoneDestination>)> = vec![
        (
            "csr-calculation",
            Box::new(CsrCalculationSink {
                client: certificates::csr_calculation_service_client::CsrCalculationServiceClient::new(channel.clone()),
            }),
        ),
        (
            "onboarding-plan-calculation",
            Box::new(OnboardingPlanCalculationSink {
                client:
                    onboarding::onboarding_plan_calculation_service_client::OnboardingPlanCalculationServiceClient::new(
                        channel.clone(),
                    ),
            }),
        ),
        (
            "vehicle-query",
            Box::new(VehicleQuerySink {
                client: fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "csr-calculation" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `mirror_sponsors_timezone` keeps of each SummarizeSponsors response.
#[derive(Clone, Debug, Default)]
pub struct SponsorsTimezoneRecord {
    pub timezone: String,
    pub report_id: String,
}

/// Where `mirror_sponsors_timezone` hands each record.
trait SponsorsTimezoneDestination {
    fn push(&self, record: SponsorsTimezoneRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CsrCalculationService`.
struct CsrCalculationSink {
    client: certificates::csr_calculation_service_client::CsrCalculationServiceClient<Channel>,
}

impl SponsorsTimezoneDestination for CsrCalculationSink {
    fn push(&self, record: SponsorsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_csr(certificates::SimulateCsrRequest {
                    csr: Some(certificates::Csr {
                        subject: record.timezone,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.onboarding.v1.OnboardingPlanCalculationService`.
struct OnboardingPlanCalculationSink {
    client: onboarding::onboarding_plan_calculation_service_client::OnboardingPlanCalculationServiceClient<Channel>,
}

impl SponsorsTimezoneDestination for OnboardingPlanCalculationSink {
    fn push(&self, record: SponsorsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_onboarding_plan(onboarding::QuoteOnboardingPlanRequest {
                    onboarding_plan_id: record.report_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.fleet.v1.VehicleQueryService`.
struct VehicleQuerySink {
    client: fleet::vehicle_query_service_client::VehicleQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl SponsorsTimezoneDestination for VehicleQuerySink {
    fn push(&self, record: SponsorsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::SummarizeVehiclesRequest {
                    details: Some(fleet::summarize_vehicles_request::Details {
                        format: record.report_id,
                        cron: record.timezone,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_vehicles(request).await?;
            Ok(())
        })
    }
}
