//! `export_flag_rollout_message` reads `acme.featureflags.v1.FlagRolloutAdminService/RepairFlagRollout` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::tours::v1 as tours;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient::new(channel.clone());
    let reply = source
        .repair_flag_rollout(featureflags::RepairFlagRolloutRequest {
            context: Some(context.clone()),
            reason: "default-export-flag-rollout-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = FlagRolloutMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        outcome_message: outcome.message,
    };
    let mut sinks: BTreeMap<String, Box<dyn FlagRolloutMessageSink>> = BTreeMap::new();
    sinks.insert(
        "role-binding-notification".to_owned(),
        Box::new(RoleBindingNotificationSink {
            client: authorization::role_binding_notification_service_client::RoleBindingNotificationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "admission-calculation".to_owned(),
        Box::new(AdmissionCalculationSink {
            client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "concession-stand".to_owned(),
        Box::new(ConcessionStandSink {
            client: concessions::concession_stand_service_client::ConcessionStandServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "tour-query".to_owned(),
        Box::new(TourQuerySink {
            client: tours::tour_query_service_client::TourQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let target = std::env::var("EXPORT_FLAG_ROLLOUT_MESSAGE_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.forward(record).await,
        None => Ok(()),
    }
}

/// What `export_flag_rollout_message` keeps of each RepairFlagRollout response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutMessageRecord {
    pub message: String,
    pub outcome_message: String,
}

/// Where `export_flag_rollout_message` hands each record.
trait FlagRolloutMessageSink {
    fn forward(&self, record: FlagRolloutMessageRecord) -> Delivery;
}

/// Writes records to `acme.authorization.v1.RoleBindingNotificationService`.
struct RoleBindingNotificationSink {
    client: authorization::role_binding_notification_service_client::RoleBindingNotificationServiceClient<Channel>,
}

impl FlagRolloutMessageSink for RoleBindingNotificationSink {
    fn forward(&self, record: FlagRolloutMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_role_binding_subscriptions(authorization::ListRoleBindingSubscriptionsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    order_by: record.outcome_message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.admissions.v1.AdmissionCalculationService`.
struct AdmissionCalculationSink {
    client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient<Channel>,
}

impl FlagRolloutMessageSink for AdmissionCalculationSink {
    fn forward(&self, record: FlagRolloutMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_admissions(admissions::RecalculateAdmissionsRequest {
                    reason: record.message,
                    admission: Some(admissions::Admission {
                        kind: record.outcome_message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.concessions.v1.ConcessionStandService`.
struct ConcessionStandSink {
    client: concessions::concession_stand_service_client::ConcessionStandServiceClient<Channel>,
}

impl FlagRolloutMessageSink for ConcessionStandSink {
    fn forward(&self, record: FlagRolloutMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_concession_stands(concessions::SearchConcessionStandsRequest {
                    filter: Some(concessions::search_concession_stands_request::Filter {
                        region: record.outcome_message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.tours.v1.TourQueryService`.
struct TourQuerySink {
    client: tours::tour_query_service_client::TourQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagRolloutMessageSink for TourQuerySink {
    fn forward(&self, record: FlagRolloutMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(tours::FindToursRequest {
                    page: Some(common::PageRequest {
                        page_token: record.outcome_message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.find_tours(request).await?;
            Ok(())
        })
    }
}
