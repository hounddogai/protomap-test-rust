//! `backfill_permission_check_id` reads
//! `acme.authorization.v1.PermissionCheckValidationService/ValidatePermissionCheck` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::treasury::v1 as treasury;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::permission_check_validation_service_client::PermissionCheckValidationServiceClient::new(
            channel.clone(),
        );
    let reply = source
        .validate_permission_check(authorization::ValidatePermissionCheckRequest {
            context: Some(context.clone()),
            reason: "default-backfill-permission-check-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let permission_check = reply.permission_check.unwrap_or_default();
    let record = PermissionCheckIdRecord {
        reference_id: outcome.reference_id,
        status: outcome.status,
        source: permission_check.source,
        permission_check_id: permission_check.permission_check_id,
    };
    act_on_permission_check_id("flag-evaluation-calculation", record, channel, context).await
}

/// What `backfill_permission_check_id` keeps of each ValidatePermissionCheck response.
#[derive(Clone, Debug, Default)]
pub struct PermissionCheckIdRecord {
    pub reference_id: String,
    pub status: String,
    pub source: String,
    pub permission_check_id: String,
}

/// Where `backfill_permission_check_id` hands each record.
trait PermissionCheckIdSink {
    fn forward(&self, record: PermissionCheckIdRecord) -> Delivery;
}

/// Sends records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl PermissionCheckIdSink for AccessGrantValidationSink {
    fn forward(&self, record: PermissionCheckIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::CheckAccessGrantEligibilityRequest {
                    access_grant_id: record.status,
                    reason: record.permission_check_id,
                    access_grant: Some(secrets::AccessGrant {
                        access_grant_id: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.check_access_grant_eligibility(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationCalculationService`.
struct FlagEvaluationCalculationSink {
    client: featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient<Channel>,
}

impl PermissionCheckIdSink for FlagEvaluationCalculationSink {
    fn forward(&self, record: PermissionCheckIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_flag_evaluation(featureflags::CalculateFlagEvaluationRequest {
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        correlation_id: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.treasury.v1.CashPositionCalculationService`.
struct CashPositionCalculationSink {
    client: treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient<Channel>,
}

impl PermissionCheckIdSink for CashPositionCalculationSink {
    fn forward(&self, record: PermissionCheckIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_cash_position(treasury::EstimateCashPositionRequest {
                    cash_position: Some(treasury::CashPosition {
                        cash_position_id: record.permission_check_id,
                        ..Default::default()
                    }),
                    cash_position_id: record.source,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_permission_check_id(
    kind: &str,
    record: PermissionCheckIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(PermissionCheckIdRecord) -> Delivery>> = BTreeMap::new();
    let access_grant_validation_channel = channel.clone();
    let access_grant_validation_context = context.clone();
    actions.insert(
        "access-grant-validation",
        Box::new(move |record| {
            AccessGrantValidationSink {
                client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(
                    access_grant_validation_channel.clone(),
                ),
                context: access_grant_validation_context.clone(),
            }
            .forward(record)
        }),
    );
    let flag_evaluation_calculation_channel = channel.clone();
    actions.insert("flag-evaluation-calculation", Box::new(move |record| FlagEvaluationCalculationSink { client: featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient::new(flag_evaluation_calculation_channel.clone()) }.forward(record)));
    let cash_position_calculation_channel = channel.clone();
    actions.insert(
        "cash-position-calculation",
        Box::new(move |record| {
            CashPositionCalculationSink {
                client: treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient::new(
                    cash_position_calculation_channel.clone(),
                ),
            }
            .forward(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
