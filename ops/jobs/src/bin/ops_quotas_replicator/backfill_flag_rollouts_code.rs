//! `backfill_flag_rollouts_code` reads
//! `acme.featureflags.v1.FlagRolloutCalculationService/RecalculateFlagRollouts` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
        channel.clone(),
    );
    let found = source
        .recalculate_flag_rollouts(featureflags::RecalculateFlagRolloutsRequest {
            context: Some(context.clone()),
            flag_rollout_id: "primary-backfill-flag-rollouts-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let flag_rollout = found.flag_rollout.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = FlagRolloutsCodeRecord {
        currency_code: breakdown.currency_code,
        title: flag_rollout.title,
        flag_rollout_id: flag_rollout.flag_rollout_id,
        status: result.status,
    };
    act_on_flag_rollouts_code("audit-event-query", record, channel, context).await
}

/// What `backfill_flag_rollouts_code` keeps of each RecalculateFlagRollouts response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutsCodeRecord {
    pub currency_code: String,
    pub title: String,
    pub flag_rollout_id: String,
    pub status: String,
}

/// Where `backfill_flag_rollouts_code` hands each record.
trait FlagRolloutsCodeTarget {
    fn write(&self, record: FlagRolloutsCodeRecord) -> Delivery;
}

/// Hands records to `acme.giftcards.v1.GiftCardService`.
struct GiftCardSink {
    client: giftcards::gift_card_service_client::GiftCardServiceClient<Channel>,
}

impl FlagRolloutsCodeTarget for GiftCardSink {
    fn write(&self, record: FlagRolloutsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_gift_card(giftcards::DeleteGiftCardRequest {
                    gift_card_id: record.title,
                    etag: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.secrets.v1.AccessGrantCalculationService`.
struct AccessGrantCalculationSink {
    client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient<Channel>,
}

impl FlagRolloutsCodeTarget for AccessGrantCalculationSink {
    fn write(&self, record: FlagRolloutsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_access_grant(secrets::EstimateAccessGrantRequest {
                    access_grant: Some(secrets::AccessGrant {
                        description: record.flag_rollout_id,
                        access_grant_id: record.status,
                        ..Default::default()
                    }),
                    access_grant_id: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.audit.v1.AuditEventQueryService`.
struct AuditEventQuerySink {
    client: audit::audit_event_query_service_client::AuditEventQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagRolloutsCodeTarget for AuditEventQuerySink {
    fn write(&self, record: FlagRolloutsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(audit::BrowseAuditEventsRequest {
                    filter: Some(audit::browse_audit_events_request::Filter {
                        owner_id: record.currency_code,
                        query: record.title,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_audit_events(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_flag_rollouts_code(
    kind: &str,
    record: FlagRolloutsCodeRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(FlagRolloutsCodeRecord) -> Delivery>> = BTreeMap::new();
    let gift_card_channel = channel.clone();
    actions.insert(
        "gift-card",
        Box::new(move |record| {
            GiftCardSink {
                client: giftcards::gift_card_service_client::GiftCardServiceClient::new(gift_card_channel.clone()),
            }
            .write(record)
        }),
    );
    let access_grant_calculation_channel = channel.clone();
    actions.insert(
        "access-grant-calculation",
        Box::new(move |record| {
            AccessGrantCalculationSink {
                client: secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient::new(
                    access_grant_calculation_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let audit_event_query_channel = channel.clone();
    let audit_event_query_context = context.clone();
    actions.insert(
        "audit-event-query",
        Box::new(move |record| {
            AuditEventQuerySink {
                client: audit::audit_event_query_service_client::AuditEventQueryServiceClient::new(
                    audit_event_query_channel.clone(),
                ),
                context: audit_event_query_context.clone(),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
