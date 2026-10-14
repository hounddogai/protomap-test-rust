//! `sync_permission_check_rules_status` reads
//! `acme.authorization.v1.PermissionCheckValidationService/GetPermissionCheckRules` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::invoicing::v1 as invoicing;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::permission_check_validation_service_client::PermissionCheckValidationServiceClient::new(
            channel.clone(),
        );
    let found = source
        .get_permission_check_rules(authorization::GetPermissionCheckRulesRequest {
            context: Some(context.clone()),
            permission_check_id: "default-sync-permission-check-rules-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let permission_check = found.permission_check.unwrap_or_default();
    let record = PermissionCheckRulesStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        source: permission_check.source,
    };
    let mut sinks: BTreeMap<String, Box<dyn PermissionCheckRulesStatusDestination>> = BTreeMap::new();
    sinks.insert(
        "broadcast".to_owned(),
        Box::new(BroadcastSink {
            client: broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "invoice-batch".to_owned(),
        Box::new(InvoiceBatchSink {
            client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert("permission-check-notification".to_owned(), Box::new(PermissionCheckNotificationSink { client: authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(channel.clone()) }));
    sinks.insert(
        "sponsor-calculation".to_owned(),
        Box::new(SponsorCalculationSink {
            client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(channel),
        }),
    );
    let target = std::env::var("SYNC_PERMISSION_CHECK_RULES_STATUS_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.apply(record).await,
        None => Ok(()),
    }
}

/// What `sync_permission_check_rules_status` keeps of each GetPermissionCheckRules response.
#[derive(Clone, Debug, Default)]
pub struct PermissionCheckRulesStatusRecord {
    pub status: String,
    pub source: String,
}

/// Where `sync_permission_check_rules_status` hands each record.
trait PermissionCheckRulesStatusDestination {
    fn apply(&self, record: PermissionCheckRulesStatusRecord) -> Delivery;
}

/// Passes records to `acme.broadcasts.v1.BroadcastService`.
struct BroadcastSink {
    client: broadcasts::broadcast_service_client::BroadcastServiceClient<Channel>,
}

impl PermissionCheckRulesStatusDestination for BroadcastSink {
    fn apply(&self, record: PermissionCheckRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_broadcast(broadcasts::GetBroadcastRequest {
                    broadcast_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.invoicing.v1.InvoiceBatchService`.
struct InvoiceBatchSink {
    client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient<Channel>,
}

impl PermissionCheckRulesStatusDestination for InvoiceBatchSink {
    fn apply(&self, record: PermissionCheckRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_invoice_batch(invoicing::GetInvoiceBatchRequest {
                    invoice_batch_id: record.source,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.PermissionCheckNotificationService`.
struct PermissionCheckNotificationSink {
    client:
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient<Channel>,
}

impl PermissionCheckRulesStatusDestination for PermissionCheckNotificationSink {
    fn apply(&self, record: PermissionCheckRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_permission_check_subscriptions(authorization::ListPermissionCheckSubscriptionsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.sponsorships.v1.SponsorCalculationService`.
struct SponsorCalculationSink {
    client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient<Channel>,
}

impl PermissionCheckRulesStatusDestination for SponsorCalculationSink {
    fn apply(&self, record: PermissionCheckRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_sponsor(sponsorships::QuoteSponsorRequest {
                    reason: record.status,
                    details: Some(sponsorships::quote_sponsor_request::Details {
                        currency_code: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
