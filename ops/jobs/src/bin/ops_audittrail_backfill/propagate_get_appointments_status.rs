//! `propagate_get_appointments_status` reads `acme.appointments.v1.AppointmentService/BatchGetAppointments`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::checkout::v1 as checkout;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = appointments::appointment_service_client::AppointmentServiceClient::new(channel.clone());
    let loaded = source
        .batch_get_appointments(appointments::BatchGetAppointmentsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = GetAppointmentsStatusRecord {
        status: result.status,
        message: result.message,
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
    };
    let sinks: Vec<(&str, Box<dyn GetAppointmentsStatusDestination>)> = vec![
        (
            "config-set-approval",
            Box::new(ConfigSetApprovalSink {
                client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "box-office-sale",
            Box::new(BoxOfficeSaleSink {
                client: boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient::new(channel.clone()),
            }),
        ),
        (
            "seller-query",
            Box::new(SellerQuerySink {
                client: sellers::seller_query_service_client::SellerQueryServiceClient::new(channel.clone()),
                context: context.clone(),
            }),
        ),
        (
            "checkout-session-query",
            Box::new(CheckoutSessionQuerySink {
                client: checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "checkout-session-query" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `propagate_get_appointments_status` keeps of each BatchGetAppointments response.
#[derive(Clone, Debug, Default)]
pub struct GetAppointmentsStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
    pub next_page_token: String,
}

/// Where `propagate_get_appointments_status` hands each record.
trait GetAppointmentsStatusDestination {
    fn apply(&self, record: GetAppointmentsStatusRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.ConfigSetApprovalService`.
struct ConfigSetApprovalSink {
    client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetAppointmentsStatusDestination for ConfigSetApprovalSink {
    fn apply(&self, record: GetAppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::DelegateConfigSetApprovalRequest {
                    details: Some(configuration::delegate_config_set_approval_request::Details {
                        approver_email: record.correlation_id,
                        ..Default::default()
                    }),
                    config_set: Some(configuration::ConfigSet {
                        name: record.next_page_token,
                        ..Default::default()
                    }),
                    config_set_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delegate_config_set_approval(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.boxoffice.v1.BoxOfficeSaleService`.
struct BoxOfficeSaleSink {
    client: boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient<Channel>,
}

impl GetAppointmentsStatusDestination for BoxOfficeSaleSink {
    fn apply(&self, record: GetAppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_box_office_sale(boxoffice::DeleteBoxOfficeSaleRequest {
                    box_office_sale_id: record.message,
                    etag: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.sellers.v1.SellerQueryService`.
struct SellerQuerySink {
    client: sellers::seller_query_service_client::SellerQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetAppointmentsStatusDestination for SellerQuerySink {
    fn apply(&self, record: GetAppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sellers::BrowseSellersRequest {
                    page: Some(common::PageRequest {
                        page_token: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_sellers(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.checkout.v1.CheckoutSessionQueryService`.
struct CheckoutSessionQuerySink {
    client: checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient<Channel>,
}

impl GetAppointmentsStatusDestination for CheckoutSessionQuerySink {
    fn apply(&self, record: GetAppointmentsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_checkout_sessions(checkout::CountCheckoutSessionsRequest {
                    order_by: record.correlation_id,
                    filter: Some(checkout::count_checkout_sessions_request::Filter {
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
