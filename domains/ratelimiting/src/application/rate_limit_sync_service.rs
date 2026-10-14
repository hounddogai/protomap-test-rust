use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient;
use acme_proto::acme::configuration::v1::config_set_workflow_service_client::ConfigSetWorkflowServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_admin_service_client::FeatureFlagAdminServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_client::RateLimitCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_client::RateLimitQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_sync_service_server::RateLimitSyncService;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_query_service_client::SearchIndexQueryServiceClient;

/// Keeps rate limits in step with the systems of record in other regions.
#[derive(Clone)]
pub struct RateLimitSyncApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
}
impl RateLimitSyncApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SyncRateLimits after validating the tenant and durably recording the command.
    async fn run_sync_rate_limits(
        &self,
        req: ratelimiting::SyncRateLimitsRequest,
    ) -> Result<ratelimiting::SyncRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SyncRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SyncRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::EstimateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: req
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
                        percent: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .percent
                        .clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    user_id: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .rate_limit
            .clone()
            .unwrap_or_default()
            .conditions
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = ratelimiting::CreateRateLimitRequest {
                        context: req.context.clone(),
                        rate_limit: Some(ratelimiting::RateLimit {
                            conditions: vec![ratelimiting::RateLimitCondition {
                                attribute: item.attribute.clone(),
                                negate: item.negate.clone(),
                                operator: item.operator.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = RateLimitServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::CreateRateLimitRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.create_rate_limit(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = ratelimiting::SyncRateLimitsResponse::default();
        let contribution: Vec<String> = reply_0.rate_limit.clone().unwrap_or_default().scopes.clone();
        if contribution != <Vec<String>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .scopes
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = req.rate_limit.clone().unwrap_or_default().description.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).description = contribution;
        }
        Ok(response)
    }
    /// Run PushRateLimit after validating the tenant and durably recording the command.
    async fn run_push_rate_limit(
        &self,
        req: ratelimiting::PushRateLimitRequest,
    ) -> Result<ratelimiting::PushRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PushRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PushRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ApproveConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::approve_config_change_request::Details {
                approver: Some(identity::UserRef {
                    display_name: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                        Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                        _ => Default::default(),
                    })
                    .notify_email
                    .clone(),
                    user_id: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                approver_email: req
                    .rate_limit
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
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeApprovalServiceClient::new(channel);
            let input: tonic::Request<configuration::ApproveConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.approve_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ValidateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    email: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(ratelimiting::RateLimitRef {
                    display_name: reply_0
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: req.rate_limit.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ValidateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::RepairFeatureFlagRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: reply_1
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                action: Some(featureflags::feature_flag::Action::Discount(
                    featureflags::FeatureFlagDiscountAction {
                        percent: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .percent
                        .clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    email: req
                        .rate_limit
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FeatureFlagAdminServiceClient::new(channel);
            let input: tonic::Request<featureflags::RepairFeatureFlagRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.repair_feature_flag(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::PushRateLimitResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_1.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Vec<String> = reply_2.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.rate_limit.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run PullRateLimits after validating the tenant and durably recording the command.
    async fn run_pull_rate_limits(
        &self,
        req: ratelimiting::PullRateLimitsRequest,
    ) -> Result<ratelimiting::PullRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PullRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PullRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ExplainQuotaBucketDecisionRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_quota_bucket_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .rate_limit
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
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_at: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: req
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainQuotaBucketDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_quota_bucket_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::PullRateLimitsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.rate_limit.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run ReconcileRateLimits after validating the tenant and durably recording the command.
    async fn run_reconcile_rate_limits(
        &self,
        req: ratelimiting::ReconcileRateLimitsRequest,
    ) -> Result<ratelimiting::ReconcileRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReconcileRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReconcileRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::RetryConfigChangeWorkflowRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
                        .rate_limit
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
            details: Some(configuration::retry_config_change_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    email: req
                        .rate_limit
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::ExplainQuotaBucketDecisionRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                limits: vec![ratelimiting::QuotaBucketLimit {
                    amount: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                        Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                        _ => Default::default(),
                    })
                    .amount
                    .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeWorkflowServiceClient::new(channel_0);
                let input: tonic::Request<configuration::RetryConfigChangeWorkflowRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.retry_config_change_workflow(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketValidationServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::ExplainQuotaBucketDecisionRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_quota_bucket_decision(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::ReconcileRateLimitsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i64 = req.rate_limit.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        Ok(response)
    }
    /// Run StreamRateLimitChanges after validating the tenant and durably recording the command.
    async fn run_stream_rate_limit_changes(
        &self,
        req: ratelimiting::StreamRateLimitChangesRequest,
    ) -> Result<ratelimiting::StreamRateLimitChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("StreamRateLimitChanges", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamRateLimitChanges", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value, _ => Default::default() }).description.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CheckQuotaBucketEligibilityRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    etag: (match req.frame.clone() {
                        Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .etag
                    .clone(),
                    updated_at: (match req.frame.clone() {
                        Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                    updated_by: (match req.frame.clone() {
                        Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_by
                    .clone(),
                    ..Default::default()
                }),
                holder: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: (match req.frame.clone() {
                        Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .rate_limit_id
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CheckQuotaBucketEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_quota_bucket_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::StreamConfigChangeMetricsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: (match req.frame.clone() {
                    Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .owner
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
                name: reply_0
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: reply_0
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::StreamConfigChangeMetricsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_config_change_metrics(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::StreamConfigChangeMetricsResponse::default();
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
        let outgoing_2 = configuration::StartConfigSetWorkflowRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        updated_at: (match req.frame.clone() {
                            Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                            _ => Default::default(),
                        })
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                        ..Default::default()
                    }),
                    children: vec![configuration::ConfigSet {
                        children: vec![configuration::ConfigSet {
                            name: (match req.frame.clone() {
                                Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                                _ => Default::default(),
                            })
                            .name
                            .clone(),
                            ..Default::default()
                        }],
                        manager: Some(identity::UserRef {
                            display_name: reply_0
                                .quota_bucket
                                .clone()
                                .unwrap_or_default()
                                .holder
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: (match req.frame.clone() {
                                Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
                                _ => Default::default(),
                            })
                            .owner
                            .clone()
                            .unwrap_or_default()
                            .email
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
        let reply_2 = async move {
            let mut client = ConfigSetWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::StartConfigSetWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.start_config_set_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::StreamRateLimitChangesResponse::default();
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
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = reply_2
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = (match req.frame.clone() {
            Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .rate_limit_id
        .clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> = (match req.frame.clone() {
            Some(ratelimiting::stream_rate_limit_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .parameters
        .clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ResolveRateLimitConflict after validating the tenant and durably recording the command.
    async fn run_resolve_rate_limit_conflict(
        &self,
        req: ratelimiting::ResolveRateLimitConflictRequest,
    ) -> Result<ratelimiting::ResolveRateLimitConflictResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveRateLimitConflict", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveRateLimitConflict", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ExportRateLimitsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::export_rate_limits_request::Filter {
                created_after: req
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.rate_limit.clone().unwrap_or_default().parameters.clone(),
                owner_id: req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = RateLimitQueryServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExportRateLimitsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_rate_limits(input).await?;
            let mut frames = response.into_inner();
            let mut last = ratelimiting::ExportRateLimitsResponse::default();
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
        let outgoing_1 = ratelimiting::ExplainThrottleDecisionRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_throttle_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .rate_limit
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ThrottleValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainThrottleDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_throttle_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ResolveRateLimitConflictResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.rate_limit.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.rate_limit.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run GetRateLimitSyncState after validating the tenant and durably recording the command.
    async fn run_get_rate_limit_sync_state(
        &self,
        req: ratelimiting::GetRateLimitSyncStateRequest,
    ) -> Result<ratelimiting::GetRateLimitSyncStateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRateLimitSyncState", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRateLimitSyncState", "running workflow");
        let outgoing_0 = ratelimiting::CheckQuotaBucketEligibilityRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    email: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                holder: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: req.rate_limit_id.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CheckQuotaBucketEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_quota_bucket_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = search::SummarizeSearchIndexesRequest {
            annotations: reply_0.quota_bucket.clone().unwrap_or_default().labels.clone(),
            context: req.context.clone(),
            details: Some(search::summarize_search_indexes_request::Details {
                filters: reply_0.result.clone().unwrap_or_default().details.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SearchIndexQueryServiceClient::new(channel);
            let input: tonic::Request<search::SummarizeSearchIndexesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::GetRateLimitSyncStateResponse::default();
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
            response.rate_limit.get_or_insert_with(Default::default).version = contribution;
        }
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
    /// Run ListRateLimitConflicts after validating the tenant and durably recording the command.
    async fn run_list_rate_limit_conflicts(
        &self,
        req: ratelimiting::ListRateLimitConflictsRequest,
    ) -> Result<ratelimiting::ListRateLimitConflictsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListRateLimitConflicts", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListRateLimitConflicts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::UpdateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                action: Some(ratelimiting::rate_limit::Action::Block(
                    ratelimiting::RateLimitBlockAction {
                        notify_email: req
                            .context
                            .clone()
                            .unwrap_or_default()
                            .actor
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::UpdateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ListRateLimitConflictsResponse::default();
        let contribution: Option<common::TimeWindow> = reply_0.rate_limit.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            if response.rate_limits.is_empty() {
                response.rate_limits.push(Default::default());
            }
            response.rate_limits[0].effective = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        Ok(response)
    }
}
acme_platform::endpoints! { RateLimitSyncService, RateLimitSyncApi;
unary {
sync_rate_limits (ratelimiting::SyncRateLimitsRequest) -> ratelimiting::SyncRateLimitsResponse => run_sync_rate_limits;
push_rate_limit (ratelimiting::PushRateLimitRequest) -> ratelimiting::PushRateLimitResponse => run_push_rate_limit;
pull_rate_limits (ratelimiting::PullRateLimitsRequest) -> ratelimiting::PullRateLimitsResponse => run_pull_rate_limits;
reconcile_rate_limits (ratelimiting::ReconcileRateLimitsRequest) -> ratelimiting::ReconcileRateLimitsResponse => run_reconcile_rate_limits;
resolve_rate_limit_conflict (ratelimiting::ResolveRateLimitConflictRequest) -> ratelimiting::ResolveRateLimitConflictResponse => run_resolve_rate_limit_conflict;
get_rate_limit_sync_state (ratelimiting::GetRateLimitSyncStateRequest) -> ratelimiting::GetRateLimitSyncStateResponse => run_get_rate_limit_sync_state;
list_rate_limit_conflicts (ratelimiting::ListRateLimitConflictsRequest) -> ratelimiting::ListRateLimitConflictsResponse => run_list_rate_limit_conflicts;
}
server {
}
client {
}
bidi {
stream_rate_limit_changes (ratelimiting::StreamRateLimitChangesRequest) -> ratelimiting::StreamRateLimitChangesResponse, StreamRateLimitChangesStream => run_stream_rate_limit_changes;
}
}
