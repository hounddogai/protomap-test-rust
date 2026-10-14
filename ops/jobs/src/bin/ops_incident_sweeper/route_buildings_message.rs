//! `route_buildings_message` reads `acme.workplaces.v1.BuildingQueryService/CountBuildings` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::workplaces::v1 as workplaces;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = workplaces::building_query_service_client::BuildingQueryServiceClient::new(channel.clone());
    let found = source
        .count_buildings(workplaces::CountBuildingsRequest {
            context: Some(context.clone()),
            order_by: "live-route-buildings-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = BuildingsMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        status: result.status,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn BuildingsMessageDestination>> = vec![
        Box::new(CertificateCalculationSink {
            client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(DispatchOrderSink {
            client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(ThrottleImportSink {
            client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient::new(channel),
        }),
    ];
    sinks[0].forward(record).await
}

/// What `route_buildings_message` keeps of each CountBuildings response.
#[derive(Clone, Debug, Default)]
pub struct BuildingsMessageRecord {
    pub message: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `route_buildings_message` hands each record.
trait BuildingsMessageDestination {
    fn forward(&self, record: BuildingsMessageRecord) -> Delivery;
}

/// Writes records to `acme.certificates.v1.CertificateCalculationService`.
struct CertificateCalculationSink {
    client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl BuildingsMessageDestination for CertificateCalculationSink {
    fn forward(&self, record: BuildingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::QuoteCertificateRequest {
                    certificate: Some(certificates::Certificate {
                        number: record.message,
                        notes: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.quote_certificate(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.dispatch.v1.DispatchOrderService`.
struct DispatchOrderSink {
    client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient<Channel>,
    context: common::RequestContext,
}

impl BuildingsMessageDestination for DispatchOrderSink {
    fn forward(&self, record: BuildingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(dispatch::DeleteDispatchOrderRequest {
                    etag: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delete_dispatch_order(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleImportService`.
struct ThrottleImportSink {
    client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient<Channel>,
}

impl BuildingsMessageDestination for ThrottleImportSink {
    fn forward(&self, record: BuildingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_throttle_import(ratelimiting::ValidateThrottleImportRequest {
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
