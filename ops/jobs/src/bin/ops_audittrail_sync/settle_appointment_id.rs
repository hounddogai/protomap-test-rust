//! `settle_appointment_id` reads `acme.appointments.v1.AppointmentService/CreateAppointment` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::inventory::v1 as inventory;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone());
    let loaded = source
        .create_appointment(appointments::CreateAppointmentRequest {
            context: Some(context.clone()),
            request_id: "live-settle-appointment-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let appointment = loaded.appointment.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = AppointmentIdRecord {
        appointment_id: acme_kit::text::normalize(&appointment.appointment_id),
        correlation_id: result.correlation_id,
    };
    deliver_appointment_id("stock-item-query", record, channel, &context).await
}

/// What `settle_appointment_id` keeps of each CreateAppointment response.
#[derive(Clone, Debug, Default)]
pub struct AppointmentIdRecord {
    pub appointment_id: String,
    pub correlation_id: String,
}

/// Where `settle_appointment_id` hands each record.
trait AppointmentIdDestination {
    fn send(&self, record: AppointmentIdRecord) -> Delivery;
}

/// Writes records to `acme.secrets.v1.AccessGrantCalculationService`.
struct AccessGrantCalculationSink {
    client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient<Channel>,
}

impl AppointmentIdDestination for AccessGrantCalculationSink {
    fn send(&self, record: AppointmentIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_access_grant(secrets::QuoteAccessGrantRequest {
                    details: Some(secrets::quote_access_grant_request::Details {
                        currency_code: record.appointment_id,
                        ..Default::default()
                    }),
                    access_grant: Some(secrets::AccessGrant {
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.secrets.v1.SecretVersionApprovalService`.
struct SecretVersionApprovalSink {
    client: secrets::secret_version_approval_service_client::SecretVersionApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl AppointmentIdDestination for SecretVersionApprovalSink {
    fn send(&self, record: AppointmentIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::ListPendingSecretVersionsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    filter: Some(secrets::list_pending_secret_versions_request::Filter {
                        owner_id: record.appointment_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_pending_secret_versions(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.inventory.v1.StockItemQueryService`.
struct StockItemQuerySink {
    client: inventory::stock_item_query_service_client::StockItemQueryServiceClient<Channel>,
}

impl AppointmentIdDestination for StockItemQuerySink {
    fn send(&self, record: AppointmentIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_stock_items(inventory::FindStockItemsRequest {
                    filter: Some(inventory::find_stock_items_request::Filter {
                        region: record.correlation_id,
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
struct AppointmentIdLogOnly;

impl AppointmentIdDestination for AppointmentIdLogOnly {
    fn send(&self, record: AppointmentIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.appointment_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_appointment_id(
    kind: &str,
    record: AppointmentIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AppointmentIdDestination> = match kind {
        "access-grant-calculation" => Box::new(AccessGrantCalculationSink {
            client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        "secret-version-approval" => Box::new(SecretVersionApprovalSink {
            client: secrets::secret_version_approval_service_client::SecretVersionApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "stock-item-query" => Box::new(StockItemQuerySink {
            client: inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel),
        }),
        _ => Box::new(AppointmentIdLogOnly),
    };
    sink.send(record).await
}
