use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::broadcasts::v1::broadcast_query_service_client::BroadcastQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_workflow_service_client::CertificateWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::fulfillment::v1::fulfillment_service_client::FulfillmentServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_validation_service_client::UserValidationServiceClient;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::access_grant_validation_service_client::AccessGrantValidationServiceClient;
use acme_proto::acme::secrets::v1::secret_approval_service_client::SecretApprovalServiceClient;
use acme_proto::acme::secrets::v1::secret_service_client::SecretServiceClient;
use acme_proto::acme::secrets::v1::secret_version_workflow_service_server::SecretVersionWorkflowService;
use acme_proto::acme::secrets::v1::secret_workflow_service_client::SecretWorkflowServiceClient;

/// Runs the long-lived workflow that takes a secret version from start to finish.
#[derive(Clone)]
pub struct SecretVersionWorkflowEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    cancel_secret_version_workflow_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<secrets::ScoreAccessGrantRequest, secrets::ScoreAccessGrantResponse>,
    >,
    list_secret_version_workflows_0_port:
        acme_platform::ports::Callback<secrets::RetrySecretWorkflowRequest, secrets::RetrySecretWorkflowResponse>,
    start_secret_version_workflow_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<secrets::LintAccessGrantsRequest, secrets::LintAccessGrantsResponse>,
    >,
    start_secret_version_workflow_2_port:
        acme_platform::ports::Callback<broadcasts::CountBroadcastsRequest, broadcasts::CountBroadcastsResponse>,
}
impl SecretVersionWorkflowEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            secrets::LintAccessGrantsRequest,
            secrets::LintAccessGrantsResponse,
        > = {
            let client = AccessGrantValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::LintAccessGrantsRequest| -> acme_platform::ports::RpcFuture<secrets::LintAccessGrantsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_access_grants(request).await?.into_inner())
})
})
        };
        let start_secret_version_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            broadcasts::CountBroadcastsRequest,
            broadcasts::CountBroadcastsResponse,
        > = {
            let client = BroadcastQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: broadcasts::CountBroadcastsRequest| -> acme_platform::ports::RpcFuture<broadcasts::CountBroadcastsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_broadcasts(request).await?.into_inner())
})
})
        };
        let start_secret_version_workflow_2_port = callback;
        let callback: acme_platform::ports::Callback<
            secrets::ScoreAccessGrantRequest,
            secrets::ScoreAccessGrantResponse,
        > = {
            let client = AccessGrantValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::ScoreAccessGrantRequest| -> acme_platform::ports::RpcFuture<secrets::ScoreAccessGrantResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_access_grant(request).await?.into_inner())
})
})
        };
        let cancel_secret_version_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            secrets::RetrySecretWorkflowRequest,
            secrets::RetrySecretWorkflowResponse,
        > = {
            let client = SecretWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::RetrySecretWorkflowRequest| -> acme_platform::ports::RpcFuture<secrets::RetrySecretWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.retry_secret_workflow(request).await?.into_inner())
})
})
        };
        let list_secret_version_workflows_0_port = callback;
        Self {
            channel,
            repository,
            cancel_secret_version_workflow_0_port,
            list_secret_version_workflows_0_port,
            start_secret_version_workflow_0_port,
            start_secret_version_workflow_2_port,
        }
    }
    /// Run StartSecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_start_secret_version_workflow(
        &self,
        req: secrets::StartSecretVersionWorkflowRequest,
    ) -> Result<secrets::StartSecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartSecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartSecretVersionWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::LintAccessGrantsRequest {
            access_grant: Some(secrets::AccessGrant {
                owner: Some(identity::UserRef {
                    display_name: req
                        .secret_version
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(secrets::lint_access_grants_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.start_secret_version_workflow_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = secrets::CheckAccessGrantEligibilityRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: reply_0
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                action: Some(secrets::access_grant::Action::Discount(
                    secrets::AccessGrantDiscountAction {
                        percent: (match reply_0.access_grant.clone().unwrap_or_default().action.clone() {
                            Some(secrets::access_grant::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .percent
                        .clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    display_name: reply_0
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessGrantValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::CheckAccessGrantEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_access_grant_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = broadcasts::CountBroadcastsRequest {
            context: req.context.clone(),
            filter: Some(broadcasts::count_broadcasts_request::Filter {
                created_after: reply_0
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_2 = (self.start_secret_version_workflow_2_port)(outgoing_2).await?;
        let mut response = secrets::StartSecretVersionWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.secret_version.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run AdvanceSecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_secret_version_workflow(
        &self,
        req: secrets::AdvanceSecretVersionWorkflowRequest,
    ) -> Result<secrets::AdvanceSecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceSecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceSecretVersionWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let jobs_0 = req
            .secret_version
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = featureflags::NormalizeFlagEvaluationRequest {
                        context: req.context.clone(),
                        flag_evaluation: Some(featureflags::FlagEvaluation {
                            measurements: vec![featureflags::FlagEvaluationMeasurement {
                                metric: item.metric.clone(),
                                observed_at: item.observed_at.clone(),
                                unit: item.unit.clone(),
                                value: item.value.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = FlagEvaluationValidationServiceClient::new(channel);
                        let input: tonic::Request<featureflags::NormalizeFlagEvaluationRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.normalize_flag_evaluation(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = identity::GetUserRulesRequest {
            context: req.context.clone(),
            user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = UserValidationServiceClient::new(channel);
            let input: tonic::Request<identity::GetUserRulesRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_user_rules(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = secrets::GetAccessGrantRatesRequest {
            access_grant_id: req
                .secret_version
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .access_grant_id
                .clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::GetAccessGrantRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_access_grant_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::AdvanceSecretVersionWorkflowResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.secret_version.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret_version.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run CancelSecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_secret_version_workflow(
        &self,
        req: secrets::CancelSecretVersionWorkflowRequest,
    ) -> Result<secrets::CancelSecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelSecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelSecretVersionWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ScoreAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                access_grant_id: req
                    .secret_version
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .access_grant_id
                    .clone(),
                action: Some(secrets::access_grant::Action::Block(secrets::AccessGrantBlockAction {
                    reason: req.reason.clone(),
                    ..Default::default()
                })),
                parameters: req.details.clone().unwrap_or_default().parameters.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.cancel_secret_version_workflow_0_port.as_ref(), outgoing_0).await?;
        let mut response = secrets::CancelSecretVersionWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: i64 = req.secret_version.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.secret_version.get_or_insert_with(Default::default).sequence = contribution;
        }
        Ok(response)
    }
    /// Run RetrySecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_secret_version_workflow(
        &self,
        req: secrets::RetrySecretVersionWorkflowRequest,
    ) -> Result<secrets::RetrySecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetrySecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetrySecretVersionWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::PullRateLimitsRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::codec::through_any(&outgoing_0, "acme.ratelimiting.v1.PullRateLimitsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PullRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.pull_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = fulfillment::WatchFulfillmentsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = FulfillmentServiceClient::new(channel);
            let input: tonic::Request<fulfillment::WatchFulfillmentsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_fulfillments(input).await?;
            let mut frames = response.into_inner();
            let mut last = fulfillment::WatchFulfillmentsResponse::default();
            // The caller asks for a bounded snapshot, then drops its subscription.
            for _ in 0..8 {
                match frames.message().await? {
                    Some(frame) => last = frame,
                    None => break,
                }
            }
            Ok::<_, tonic::Status>(last)
        }
        .await?;
        let outgoing_2 = ratelimiting::PushRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: reply_0
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                action: Some(ratelimiting::rate_limit::Action::Discount(
                    ratelimiting::RateLimitDiscountAction {
                        percent: (match reply_0.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .percent
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::PushRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = secrets::RetrySecretVersionWorkflowResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.secret_version.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PauseSecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_secret_version_workflow(
        &self,
        req: secrets::PauseSecretVersionWorkflowRequest,
    ) -> Result<secrets::PauseSecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseSecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseSecretVersionWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let jobs_0 = req
            .secret_version
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = notifications::ListNotificationsRequest {
                        context: req.context.clone(),
                        filter: Some(notifications::list_notifications_request::Filter {
                            created_after: item.observed_at.clone(),
                            labels: req.secret_version.clone().unwrap_or_default().attributes.clone(),
                            owner_id: req.secret_version.clone().unwrap_or_default().correlation_id.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = NotificationServiceClient::new(channel);
                        let input: tonic::Request<notifications::ListNotificationsRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.list_notifications(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = secrets::PauseSecretVersionWorkflowResponse::default();
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: String = req.secret_version.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.secret_version.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
    /// Run ResumeSecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_secret_version_workflow(
        &self,
        req: secrets::ResumeSecretVersionWorkflowRequest,
    ) -> Result<secrets::ResumeSecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeSecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeSecretVersionWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ReconcileFeatureFlagsRequest {
            context: req.context.clone(),
            details: Some(featureflags::reconcile_feature_flags_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                action: Some(featureflags::feature_flag::Action::Block(
                    featureflags::FeatureFlagBlockAction {
                        notify_email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    display_name: req
                        .secret_version
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ReconcileFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::ResumeSecretWorkflowRequest {
            context: req.context.clone(),
            details: Some(secrets::resume_secret_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: reply_0
                        .feature_flag
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<secrets::ResumeSecretWorkflowRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SecretWorkflowServiceClient::new(channel);
            let input: tonic::Request<secrets::ResumeSecretWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_secret_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = ResumeSecretVersionWorkflowDispatch2::from(req.clone());
        command_2.access_grant_audit_created_at = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        command_2.access_grant_route_weight = (match reply_0.feature_flag.clone().unwrap_or_default().action.clone() {
            Some(featureflags::feature_flag::Action::Route(value)) => value,
            _ => Default::default(),
        })
        .weight
        .clone();
        command_2.access_grant_audit_etag = reply_1
            .secret
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        let outgoing_2: secrets::EstimateAccessGrantRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ResumeSecretVersionWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.secret_version.clone().unwrap_or_default().occurred_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret_version.get_or_insert_with(Default::default).occurred_at = contribution;
        }
        Ok(response)
    }
    /// Run InspectSecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_secret_version_workflow(
        &self,
        req: secrets::InspectSecretVersionWorkflowRequest,
    ) -> Result<secrets::InspectSecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectSecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectSecretVersionWorkflow", "running workflow");
        let outgoing_0 = secrets::SearchSecretsRequest {
            context: req.context.clone(),
            filter: Some(secrets::search_secrets_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: (req.context.clone().unwrap_or_default().baggage.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req.secret_version_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SecretServiceClient::new(channel);
            let input: tonic::Request<secrets::SearchSecretsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_secrets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::InspectSecretVersionWorkflowResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        Ok(response)
    }
    /// Run ListSecretVersionWorkflows after validating the tenant and durably recording the command.
    async fn run_list_secret_version_workflows(
        &self,
        req: secrets::ListSecretVersionWorkflowsRequest,
    ) -> Result<secrets::ListSecretVersionWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListSecretVersionWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListSecretVersionWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = secrets::RetrySecretWorkflowRequest {
            context: req.context.clone(),
            secret: Some(secrets::Secret {
                holder: Some(identity::UserRef {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.list_secret_version_workflows_0_port)(outgoing_0).await?;
        let mut response = secrets::ListSecretVersionWorkflowsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run SignalSecretVersionWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_secret_version_workflow(
        &self,
        req: secrets::SignalSecretVersionWorkflowRequest,
    ) -> Result<secrets::SignalSecretVersionWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalSecretVersionWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalSecretVersionWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::RetryCertificateWorkflowRequest {
            certificate: Some(certificates::Certificate {
                counterparty: Some(certificates::CsrRef {
                    display_name: req
                        .secret_version
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                lines: vec![certificates::CertificateLine {
                    attributes: (req.secret_version.clone().unwrap_or_default().attributes.clone())
                        .into_iter()
                        .filter(|(key, _)| !key.is_empty())
                        .map(|(key, value)| (key, value.trim().to_owned()))
                        .collect(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::retry_certificate_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateWorkflowServiceClient::new(channel);
            let input: tonic::Request<certificates::RetryCertificateWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.retry_certificate_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::DelegateSecretApprovalRequest {
            context: req.context.clone(),
            details: Some(secrets::delegate_secret_approval_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            secret: Some(secrets::Secret {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SecretApprovalServiceClient::new(channel);
            let input: tonic::Request<secrets::DelegateSecretApprovalRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delegate_secret_approval(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::SignalSecretVersionWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        let contribution: String = req.secret_version.clone().unwrap_or_default().kind.clone();
        if contribution != <String>::default() {
            response.secret_version.get_or_insert_with(Default::default).kind = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ResumeSecretVersionWorkflow for EstimateAccessGrant.
struct ResumeSecretVersionWorkflowDispatch2 {
    context: Option<common::RequestContext>,
    access_grant_audit_created_at: Option<prost_types::Timestamp>,
    access_grant_route_weight: i32,
    access_grant_owner_display_name: String,
    access_grant_audit_etag: String,
}
impl From<secrets::ResumeSecretVersionWorkflowRequest> for ResumeSecretVersionWorkflowDispatch2 {
    fn from(req: secrets::ResumeSecretVersionWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            access_grant_audit_created_at: Default::default(),
            access_grant_route_weight: Default::default(),
            access_grant_owner_display_name: req
                .secret_version
                .clone()
                .unwrap_or_default()
                .subject
                .clone()
                .unwrap_or_default()
                .display_name
                .clone(),
            access_grant_audit_etag: Default::default(),
        }
    }
}
impl From<ResumeSecretVersionWorkflowDispatch2> for secrets::EstimateAccessGrantRequest {
    fn from(value: ResumeSecretVersionWorkflowDispatch2) -> Self {
        secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_at: value.access_grant_audit_created_at,
                    etag: value.access_grant_audit_etag,
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    display_name: value.access_grant_owner_display_name,
                    ..Default::default()
                }),
                action: Some(secrets::access_grant::Action::Route(secrets::AccessGrantRouteAction {
                    weight: value.access_grant_route_weight,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl SecretVersionWorkflowService for SecretVersionWorkflowEndpoint {
    /// Starts a secret version.
    async fn start_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::StartSecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::StartSecretVersionWorkflowResponse>, tonic::Status> {
        self.run_start_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a secret version.
    async fn advance_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::AdvanceSecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::AdvanceSecretVersionWorkflowResponse>, tonic::Status> {
        self.run_advance_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a secret version.
    async fn cancel_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::CancelSecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::CancelSecretVersionWorkflowResponse>, tonic::Status> {
        self.run_cancel_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a secret version.
    async fn retry_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::RetrySecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::RetrySecretVersionWorkflowResponse>, tonic::Status> {
        self.run_retry_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses a secret version.
    async fn pause_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::PauseSecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::PauseSecretVersionWorkflowResponse>, tonic::Status> {
        self.run_pause_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes a secret version.
    async fn resume_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::ResumeSecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::ResumeSecretVersionWorkflowResponse>, tonic::Status> {
        self.run_resume_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a secret version.
    async fn inspect_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::InspectSecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::InspectSecretVersionWorkflowResponse>, tonic::Status> {
        self.run_inspect_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secret versions a page at a time.
    async fn list_secret_version_workflows(
        &self,
        request: tonic::Request<secrets::ListSecretVersionWorkflowsRequest>,
    ) -> Result<tonic::Response<secrets::ListSecretVersionWorkflowsResponse>, tonic::Status> {
        self.run_list_secret_version_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals a secret version.
    async fn signal_secret_version_workflow(
        &self,
        request: tonic::Request<secrets::SignalSecretVersionWorkflowRequest>,
    ) -> Result<tonic::Response<secrets::SignalSecretVersionWorkflowResponse>, tonic::Status> {
        self.run_signal_secret_version_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
