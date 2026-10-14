use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_service_client::UserServiceClient;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_validation_service_client::LedgerAccountValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_server::QuotaBucketSyncService;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_client::RateLimitSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;

/// Keeps quota buckets in step with the systems of record in other regions.
#[derive(Clone)]
pub struct QuotaBucketSyncController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    pull_quota_buckets_1_port:
        std::sync::Arc<acme_platform::ports::Adapter<identity::ListUsersRequest, identity::ListUsersResponse>>,
    pull_quota_buckets_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<ratelimiting::CloneRateLimitRequest, ratelimiting::CloneRateLimitResponse>,
    >,
    push_quota_bucket_2_port: acme_platform::ports::Callback<
        configuration::AdvanceConfigChangeWorkflowRequest,
        configuration::AdvanceConfigChangeWorkflowResponse,
    >,
    sync_quota_buckets_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::RetryThrottleImportRequest,
            ratelimiting::RetryThrottleImportResponse,
        >,
    >,
}
impl QuotaBucketSyncController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::RetryThrottleImportRequest,
            ratelimiting::RetryThrottleImportResponse,
        > = {
            let client = ThrottleImportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::RetryThrottleImportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::RetryThrottleImportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.retry_throttle_import(request).await?.into_inner())
})
})
        };
        let sync_quota_buckets_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::AdvanceConfigChangeWorkflowRequest,
            configuration::AdvanceConfigChangeWorkflowResponse,
        > = {
            let client = ConfigChangeWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::AdvanceConfigChangeWorkflowRequest| -> acme_platform::ports::RpcFuture<configuration::AdvanceConfigChangeWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.advance_config_change_workflow(request).await?.into_inner())
})
})
        };
        let push_quota_bucket_2_port = callback;
        let callback: acme_platform::ports::Callback<identity::ListUsersRequest, identity::ListUsersResponse> = {
            let client = UserServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: identity::ListUsersRequest| -> acme_platform::ports::RpcFuture<identity::ListUsersResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_users(request).await?.into_inner())
})
})
        };
        let pull_quota_buckets_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::CloneRateLimitRequest,
            ratelimiting::CloneRateLimitResponse,
        > = {
            let client = RateLimitServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CloneRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CloneRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_rate_limit(request).await?.into_inner())
})
})
        };
        let pull_quota_buckets_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            pull_quota_buckets_1_port,
            pull_quota_buckets_2_port,
            push_quota_bucket_2_port,
            sync_quota_buckets_1_port,
        }
    }
    /// Run SyncQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_sync_quota_buckets(
        &self,
        req: ratelimiting::SyncQuotaBucketsRequest,
    ) -> Result<ratelimiting::SyncQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SyncQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SyncQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let jobs_0 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = featureflags::ScoreFlagEvaluationRequest {
                        context: req.context.clone(),
                        details: Some(featureflags::score_flag_evaluation_request::Details {
                            budget: item.amount.clone(),
                            currency_code: req.quota_bucket.clone().unwrap_or_default().currency_code.clone(),
                            ..Default::default()
                        }),
                        flag_evaluation: Some(featureflags::FlagEvaluation {
                            subject: Some(featureflags::FlagRolloutRef {
                                display_name: req
                                    .quota_bucket
                                    .clone()
                                    .unwrap_or_default()
                                    .holder
                                    .clone()
                                    .unwrap_or_default()
                                    .display_name
                                    .clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        reason: req.reason.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = FlagEvaluationValidationServiceClient::new(channel);
                        let input: tonic::Request<featureflags::ScoreFlagEvaluationRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.score_flag_evaluation(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = ratelimiting::RetryThrottleImportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::retry_throttle_import_request::Details {
                notify: Some(common::ContactInfo {
                    name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    rate_limit_id: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.sync_quota_buckets_1_port.as_ref(), outgoing_1).await?;
        let mut response = ratelimiting::SyncQuotaBucketsResponse::default();
        let contribution: i64 = reply_1
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::Money> = req.quota_bucket.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).balance = contribution;
        }
        Ok(response)
    }
    /// Run PushQuotaBucket after validating the tenant and durably recording the command.
    async fn run_push_quota_bucket(
        &self,
        req: ratelimiting::PushQuotaBucketRequest,
    ) -> Result<ratelimiting::PushQuotaBucketResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PushQuotaBucket", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PushQuotaBucket", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ReconcileRateLimitsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::reconcile_rate_limits_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_by: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
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
        let reply_0 = async move {
            let mut client = RateLimitSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ReconcileRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_rate_limits(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_1 = ledger::CheckLedgerAccountEligibilityRequest {
                        context: req.context.clone(),
                        ledger_account: Some(ledger::LedgerAccount {
                            audit: Some(common::AuditInfo {
                                updated_at: reply_0
                                    .rate_limit
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .updated_at
                                    .clone(),
                                ..Default::default()
                            }),
                            limits: vec![ledger::LedgerAccountLimit {
                                amount: item.amount.clone(),
                                name: item.name.clone(),
                                period: item.period.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = LedgerAccountValidationServiceClient::new(channel);
                        let input: tonic::Request<ledger::CheckLedgerAccountEligibilityRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.check_ledger_account_eligibility(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = configuration::AdvanceConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_1
                        .ledger_account
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::advance_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    name: reply_1
                        .ledger_account
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: reply_1
                        .ledger_account
                        .clone()
                        .unwrap_or_default()
                        .contact
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
        let port = acme_platform::ports::opaque(self.push_quota_bucket_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let outgoing_3 = ledger::NormalizeLedgerAccountRequest {
            context: req.context.clone(),
            details: Some(ledger::normalize_ledger_account_request::Details {
                contact: Some(common::ContactInfo {
                    phone: reply_1
                        .ledger_account
                        .clone()
                        .unwrap_or_default()
                        .contact
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
        let outgoing_3 = acme_platform::ports::RequestBuilder::<ledger::NormalizeLedgerAccountRequest>::default()
            .map(|draft| *draft = outgoing_3)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = LedgerAccountValidationServiceClient::new(channel);
            let input: tonic::Request<ledger::NormalizeLedgerAccountRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_ledger_account(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::PushQuotaBucketResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().reference_id.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).reference_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::Money> = req.quota_bucket.clone().unwrap_or_default().available.clone();
        if contribution != <Option<common::Money>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).available = contribution;
        }
        Ok(response)
    }
    /// Run PullQuotaBuckets after validating the tenant and durably recording the command.
    async fn run_pull_quota_buckets(
        &self,
        req: ratelimiting::PullQuotaBucketsRequest,
    ) -> Result<ratelimiting::PullQuotaBucketsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PullQuotaBuckets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PullQuotaBuckets", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::ListAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_access_policies_request::Filter {
                created_after: req
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: (req.quota_bucket.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .rate_limit_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessPolicyServiceClient::new(channel);
            let input: tonic::Request<authorization::ListAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = identity::ListUsersRequest {
            context: req.context.clone(),
            filter: Some(identity::list_users_request::Filter {
                labels: req.quota_bucket.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.pull_quota_buckets_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = ratelimiting::CloneRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    display_name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
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
        let reply_2 = acme_platform::ports::execute(self.pull_quota_buckets_2_port.as_ref(), outgoing_2).await?;
        let mut response = ratelimiting::PullQuotaBucketsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.rate_limit.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .limits
                .is_empty()
            {
                response
                    .quota_bucket
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.quota_bucket.get_or_insert_with(Default::default).limits[0].name = contribution;
        }
        let contribution: String = reply_2
            .rate_limit
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: String = req.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run StreamQuotaBucketChanges after validating the tenant and durably recording the command.
    async fn run_stream_quota_bucket_changes(
        &self,
        req: ratelimiting::StreamQuotaBucketChangesRequest,
    ) -> Result<ratelimiting::StreamQuotaBucketChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamQuotaBucketChanges", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamQuotaBucketChanges", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(value)) => value, _ => Default::default() }).holder.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ResumeConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: (match req.frame.clone() {
                                Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(value)) => value,
                                _ => Default::default(),
                            })
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                            updated_at: (match req.frame.clone() {
                                Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(value)) => value,
                                _ => Default::default(),
                            })
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::ResumeConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resume_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::RecallConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::recall_config_change_request::Details {
                approver: Some(identity::UserRef {
                    display_name: (match req.frame.clone() {
                        Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .display_name
                    .clone(),
                    email: (match req.frame.clone() {
                        Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .contact
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
            let mut client = ConfigChangeApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::RecallConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::StreamQuotaBucketChangesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = (match req.frame.clone() {
            Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .quota_bucket_id
        .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = (match req.frame.clone() {
            Some(ratelimiting::stream_quota_bucket_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .contact
        .clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run ResolveQuotaBucketConflict after validating the tenant and durably recording the command.
    async fn run_resolve_quota_bucket_conflict(
        &self,
        req: ratelimiting::ResolveQuotaBucketConflictRequest,
    ) -> Result<ratelimiting::ResolveQuotaBucketConflictResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveQuotaBucketConflict", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveQuotaBucketConflict", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let command_0 = ResolveQuotaBucketConflictDispatch0::from(req.clone());
        let outgoing_0: ratelimiting::ValidateQuotaBucketRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ValidateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = ratelimiting::PreviewQuotaBucketRequest {
                        context: req.context.clone(),
                        quota_bucket: Some(ratelimiting::QuotaBucket {
                            limits: vec![ratelimiting::QuotaBucketLimit {
                                name: item.name.clone(),
                                period: item.period.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = QuotaBucketValidationServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::PreviewQuotaBucketRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.preview_quota_bucket(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = ratelimiting::MapThrottleImportColumnsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::map_throttle_import_columns_request::Details {
                contact: Some(common::ContactInfo {
                    phone: reply_1
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: reply_1
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::MapThrottleImportColumnsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ThrottleImportServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::MapThrottleImportColumnsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.map_throttle_import_columns(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_2)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_2 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = ratelimiting::ResolveQuotaBucketConflictResponse::default();
        let contribution: i64 = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: i64 = reply_2
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.quota_bucket.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run GetQuotaBucketSyncState after validating the tenant and durably recording the command.
    async fn run_get_quota_bucket_sync_state(
        &self,
        req: ratelimiting::GetQuotaBucketSyncStateRequest,
    ) -> Result<ratelimiting::GetQuotaBucketSyncStateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetQuotaBucketSyncState", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetQuotaBucketSyncState", "running workflow");
        let mut response = ratelimiting::GetQuotaBucketSyncStateResponse::default();
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        Ok(response)
    }
    /// Run ListQuotaBucketConflicts after validating the tenant and durably recording the command.
    async fn run_list_quota_bucket_conflicts(
        &self,
        req: ratelimiting::ListQuotaBucketConflictsRequest,
    ) -> Result<ratelimiting::ListQuotaBucketConflictsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListQuotaBucketConflicts", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListQuotaBucketConflicts", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::ListEnvironmentConflictsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_environment_conflicts_request::Filter {
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentSyncServiceClient::new(channel);
            let input: tonic::Request<configuration::ListEnvironmentConflictsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_environment_conflicts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ListQuotaBucketConflictsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.page_info.clone().unwrap_or_default().next_page_token.clone();
        if contribution != <String>::default() {
            response.page_info.get_or_insert_with(Default::default).next_page_token = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ResolveQuotaBucketConflict for ValidateQuotaBucket.
struct ResolveQuotaBucketConflictDispatch0 {
    context: Option<common::RequestContext>,
    details_contact_preferred_channel: String,
    quota_bucket_bank_bic: String,
}
impl From<ratelimiting::ResolveQuotaBucketConflictRequest> for ResolveQuotaBucketConflictDispatch0 {
    fn from(req: ratelimiting::ResolveQuotaBucketConflictRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_contact_preferred_channel: req
                .quota_bucket
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .preferred_channel
                .clone(),
            quota_bucket_bank_bic: (match req.quota_bucket.clone().unwrap_or_default().funding.clone() {
                Some(ratelimiting::quota_bucket::Funding::Bank(value)) => value,
                _ => Default::default(),
            })
            .bic
            .clone(),
        }
    }
}
impl From<ResolveQuotaBucketConflictDispatch0> for ratelimiting::ValidateQuotaBucketRequest {
    fn from(value: ResolveQuotaBucketConflictDispatch0) -> Self {
        ratelimiting::ValidateQuotaBucketRequest {
            context: value.context,
            details: Some(ratelimiting::validate_quota_bucket_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: value.details_contact_preferred_channel,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                funding: Some(ratelimiting::quota_bucket::Funding::Bank(
                    ratelimiting::QuotaBucketBankFunding {
                        bic: value.quota_bucket_bank_bic,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl QuotaBucketSyncService for QuotaBucketSyncController {
    /// Syncs a quota bucket.
    async fn sync_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::SyncQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::SyncQuotaBucketsResponse>, tonic::Status> {
        self.run_sync_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pushs a quota bucket.
    async fn push_quota_bucket(
        &self,
        request: tonic::Request<ratelimiting::PushQuotaBucketRequest>,
    ) -> Result<tonic::Response<ratelimiting::PushQuotaBucketResponse>, tonic::Status> {
        self.run_push_quota_bucket(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pulls a quota bucket.
    async fn pull_quota_buckets(
        &self,
        request: tonic::Request<ratelimiting::PullQuotaBucketsRequest>,
    ) -> Result<tonic::Response<ratelimiting::PullQuotaBucketsResponse>, tonic::Status> {
        self.run_pull_quota_buckets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type StreamQuotaBucketChangesStream =
        tokio_stream::wrappers::ReceiverStream<Result<ratelimiting::StreamQuotaBucketChangesResponse, tonic::Status>>;
    /// Exchanges changes of quota buckets with a peer.
    async fn stream_quota_bucket_changes(
        &self,
        request: tonic::Request<tonic::Streaming<ratelimiting::StreamQuotaBucketChangesRequest>>,
    ) -> Result<tonic::Response<Self::StreamQuotaBucketChangesStream>, tonic::Status> {
        let handler = self.clone();
        let mut incoming = request.into_inner();
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let received = tokio::select! { _ = sender.closed() => break, value = incoming.message() => value };
                let result = match received {
                    Ok(Some(request)) => handler.run_stream_quota_bucket_changes(request).await,
                    Ok(None) => break,
                    Err(error) => Err(error),
                };
                let failed = result.is_err();
                if sender.send(result).await.is_err() || failed {
                    break;
                }
            }
        });
        Ok(tonic::Response::new(tokio_stream::wrappers::ReceiverStream::new(
            receiver,
        )))
    }
    /// Resolves a quota bucket.
    async fn resolve_quota_bucket_conflict(
        &self,
        request: tonic::Request<ratelimiting::ResolveQuotaBucketConflictRequest>,
    ) -> Result<tonic::Response<ratelimiting::ResolveQuotaBucketConflictResponse>, tonic::Status> {
        self.run_resolve_quota_bucket_conflict(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a quota bucket.
    async fn get_quota_bucket_sync_state(
        &self,
        request: tonic::Request<ratelimiting::GetQuotaBucketSyncStateRequest>,
    ) -> Result<tonic::Response<ratelimiting::GetQuotaBucketSyncStateResponse>, tonic::Status> {
        self.run_get_quota_bucket_sync_state(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists quota buckets a page at a time.
    async fn list_quota_bucket_conflicts(
        &self,
        request: tonic::Request<ratelimiting::ListQuotaBucketConflictsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListQuotaBucketConflictsResponse>, tonic::Status> {
        self.run_list_quota_bucket_conflicts(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
