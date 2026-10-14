//! `push_feature_flag_id` reads `acme.featureflags.v1.FeatureFlagAdminService/TransferFeatureFlag` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient::new(channel.clone());
    let reply = source
        .transfer_feature_flag(featureflags::TransferFeatureFlagRequest {
            context: Some(context.clone()),
            feature_flag_id: "default-push-feature-flag-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let feature_flag = reply.feature_flag.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = FeatureFlagIdRecord {
        feature_flag_id: acme_kit::text::normalize(&feature_flag.feature_flag_id),
        correlation_id: result.correlation_id,
        message: outcome.message,
    };
    let sinks: Vec<Box<dyn FeatureFlagIdSink>> = vec![
        Box::new(VenueSink {
            client: venues::venue_service_client::VenueServiceClient::new(channel.clone()),
        }),
        Box::new(TicketSink {
            client: helpdesk::ticket_service_client::TicketServiceClient::new(channel.clone()),
        }),
        Box::new(RoleBindingExportSink {
            client: authorization::role_binding_export_service_client::RoleBindingExportServiceClient::new(channel),
        }),
    ];
    sinks[0].write(record).await
}

/// What `push_feature_flag_id` keeps of each TransferFeatureFlag response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagIdRecord {
    pub feature_flag_id: String,
    pub correlation_id: String,
    pub message: String,
}

/// Where `push_feature_flag_id` hands each record.
trait FeatureFlagIdSink {
    fn write(&self, record: FeatureFlagIdRecord) -> Delivery;
}

/// Writes records to `acme.venues.v1.VenueService`.
struct VenueSink {
    client: venues::venue_service_client::VenueServiceClient<Channel>,
}

impl FeatureFlagIdSink for VenueSink {
    fn write(&self, record: FeatureFlagIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_venues(venues::SearchVenuesRequest {
                    order_by: record.correlation_id,
                    filter: Some(venues::search_venues_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.helpdesk.v1.TicketService`.
struct TicketSink {
    client: helpdesk::ticket_service_client::TicketServiceClient<Channel>,
}

impl FeatureFlagIdSink for TicketSink {
    fn write(&self, record: FeatureFlagIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_ticket(helpdesk::CreateTicketRequest {
                    ticket: Some(helpdesk::Ticket {
                        ticket_id: record.feature_flag_id,
                        subject: record.message,
                        ..Default::default()
                    }),
                    request_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.RoleBindingExportService`.
struct RoleBindingExportSink {
    client: authorization::role_binding_export_service_client::RoleBindingExportServiceClient<Channel>,
}

impl FeatureFlagIdSink for RoleBindingExportSink {
    fn write(&self, record: FeatureFlagIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_role_binding_exports(authorization::ListRoleBindingExportsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    filter: Some(authorization::list_role_binding_exports_request::Filter {
                        owner_id: record.message,
                        region: record.feature_flag_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
