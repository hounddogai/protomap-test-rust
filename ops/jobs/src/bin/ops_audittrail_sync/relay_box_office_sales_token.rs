//! `relay_box_office_sales_token` reads `acme.boxoffice.v1.BoxOfficeSaleQueryService/BrowseBoxOfficeSales`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(channel.clone());
    let loaded = source
        .browse_box_office_sales(boxoffice::BrowseBoxOfficeSalesRequest {
            context: Some(context.clone()),
            order_by: "nightly-relay-box-office-sales-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = loaded.page_info.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = BoxOfficeSalesTokenRecord {
        next_page_token: page_info.next_page_token,
        message: result.message,
        correlation_id: result.correlation_id,
    };
    deliver_box_office_sales_token("key-rotation-workflow", record, channel, &context).await
}

/// What `relay_box_office_sales_token` keeps of each BrowseBoxOfficeSales response.
#[derive(Clone, Debug, Default)]
pub struct BoxOfficeSalesTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `relay_box_office_sales_token` hands each record.
trait BoxOfficeSalesTokenWriter {
    fn deliver(&self, record: BoxOfficeSalesTokenRecord) -> Delivery;
}

/// Writes records to `acme.appointments.v1.AppointmentService`.
struct AppointmentSink {
    client: appointments::appointment_service_client::AppointmentServiceClient<Channel>,
}

impl BoxOfficeSalesTokenWriter for AppointmentSink {
    fn deliver(&self, record: BoxOfficeSalesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_appointment(appointments::DeleteAppointmentRequest {
                    appointment_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.customers.v1.CustomerQueryService`.
struct CustomerQuerySink {
    client: customers::customer_query_service_client::CustomerQueryServiceClient<Channel>,
}

impl BoxOfficeSalesTokenWriter for CustomerQuerySink {
    fn deliver(&self, record: BoxOfficeSalesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_customers(customers::FindCustomersRequest {
                    filter: Some(customers::find_customers_request::Filter {
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

/// Passes records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl BoxOfficeSalesTokenWriter for KeyRotationWorkflowSink {
    fn deliver(&self, record: BoxOfficeSalesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_key_rotation_workflow(keymanagement::AdvanceKeyRotationWorkflowRequest {
                    reason: record.message,
                    details: Some(keymanagement::advance_key_rotation_workflow_request::Details {
                        step: record.correlation_id,
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
struct BoxOfficeSalesTokenLogOnly;

impl BoxOfficeSalesTokenWriter for BoxOfficeSalesTokenLogOnly {
    fn deliver(&self, record: BoxOfficeSalesTokenRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.next_page_token);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_box_office_sales_token(
    kind: &str,
    record: BoxOfficeSalesTokenRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn BoxOfficeSalesTokenWriter> = match kind {
        "appointment" => Box::new(AppointmentSink {
            client: appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone()),
        }),
        "customer-query" => Box::new(CustomerQuerySink {
            client: customers::customer_query_service_client::CustomerQueryServiceClient::new(channel.clone()),
        }),
        "key-rotation-workflow" => Box::new(KeyRotationWorkflowSink {
            client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(channel),
        }),
        _ => Box::new(BoxOfficeSalesTokenLogOnly),
    };
    sink.deliver(record).await
}
