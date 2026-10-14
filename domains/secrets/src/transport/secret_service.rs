use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_admin_service_client::PermissionCheckAdminServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_client::FlagRolloutQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_service_client::CryptoKeyServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::access_grant_validation_service_client::AccessGrantValidationServiceClient;
use acme_proto::acme::secrets::v1::secret_query_service_client::SecretQueryServiceClient;
use acme_proto::acme::secrets::v1::secret_service_server::SecretService;
use acme_proto::acme::secrets::v1::secret_version_query_service_client::SecretVersionQueryServiceClient;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;

/// Manages secrets: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct SecretEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    batch_create_secrets_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ExplainConfigChangeCalculationRequest,
            configuration::ExplainConfigChangeCalculationResponse,
        >,
    >,
    clone_secret_0_port:
        acme_platform::ports::Callback<secrets::SimulateAccessGrantRequest, secrets::SimulateAccessGrantResponse>,
    list_secrets_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<secrets::BrowseSecretVersionsRequest, secrets::BrowseSecretVersionsResponse>,
    >,
    search_secrets_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                featureflags::CheckFeatureFlagEligibilityRequest,
                featureflags::CheckFeatureFlagEligibilityResponse,
            >,
    >,
    update_secret_0_port:
        acme_platform::ports::Callback<secrets::ScoreAccessGrantRequest, secrets::ScoreAccessGrantResponse>,
    watch_secrets_0_port:
        acme_platform::ports::Callback<secrets::QuoteAccessGrantRequest, secrets::QuoteAccessGrantResponse>,
}
impl SecretEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
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
        let update_secret_0_port = callback;
        let callback: acme_platform::ports::Callback<
            secrets::BrowseSecretVersionsRequest,
            secrets::BrowseSecretVersionsResponse,
        > = {
            let client = SecretVersionQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::BrowseSecretVersionsRequest| -> acme_platform::ports::RpcFuture<secrets::BrowseSecretVersionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_secret_versions(request).await?.into_inner())
})
})
        };
        let list_secrets_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::CheckFeatureFlagEligibilityRequest,
            featureflags::CheckFeatureFlagEligibilityResponse,
        > = {
            let client = FeatureFlagValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CheckFeatureFlagEligibilityRequest| -> acme_platform::ports::RpcFuture<featureflags::CheckFeatureFlagEligibilityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.check_feature_flag_eligibility(request).await?.into_inner())
})
})
        };
        let search_secrets_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            secrets::QuoteAccessGrantRequest,
            secrets::QuoteAccessGrantResponse,
        > = {
            let client = AccessGrantCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::QuoteAccessGrantRequest| -> acme_platform::ports::RpcFuture<secrets::QuoteAccessGrantResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_access_grant(request).await?.into_inner())
})
})
        };
        let watch_secrets_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::ExplainConfigChangeCalculationRequest,
            configuration::ExplainConfigChangeCalculationResponse,
        > = {
            let client = ConfigChangeCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainConfigChangeCalculationRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainConfigChangeCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_config_change_calculation(request).await?.into_inner())
})
})
        };
        let batch_create_secrets_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            secrets::SimulateAccessGrantRequest,
            secrets::SimulateAccessGrantResponse,
        > = {
            let client = AccessGrantCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::SimulateAccessGrantRequest| -> acme_platform::ports::RpcFuture<secrets::SimulateAccessGrantResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_access_grant(request).await?.into_inner())
})
})
        };
        let clone_secret_0_port = callback;
        Self {
            channel,
            repository,
            batch_create_secrets_1_port,
            clone_secret_0_port,
            list_secrets_0_port,
            search_secrets_0_port,
            update_secret_0_port,
            watch_secrets_0_port,
        }
    }
    /// Run CreateSecret after validating the tenant and durably recording the command.
    async fn run_create_secret(
        &self,
        req: secrets::CreateSecretRequest,
    ) -> Result<secrets::CreateSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let command_0 = CreateSecretDispatch0::from(req.clone());
        let outgoing_0: configuration::ExplainEnvironmentDecisionRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = EnvironmentValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainEnvironmentDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_environment_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::ExplainSecretVersionDecisionRequest {
            context: req.context.clone(),
            details: Some(secrets::explain_secret_version_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .secret
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
            secret_version: Some(secrets::SecretVersion {
                measurements: vec![secrets::SecretVersionMeasurement {
                    unit: reply_0
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .unit
                        .clone(),
                    ..Default::default()
                }],
                subject: Some(secrets::AccessGrantRef {
                    display_name: req
                        .secret
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.secrets.v1.ExplainSecretVersionDecisionRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SecretVersionValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::ExplainSecretVersionDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_secret_version_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = secrets::LintAccessGrantsRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_by: req
                        .secret
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
                    display_name: req
                        .secret
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
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<secrets::LintAccessGrantsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = AccessGrantValidationServiceClient::new(channel);
                let input: tonic::Request<secrets::LintAccessGrantsRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.lint_access_grants(input).await?;
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
        let mut response = secrets::CreateSecretResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: i32 = req.secret.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.secret.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::Money> = req.secret.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.secret.get_or_insert_with(Default::default).balance = contribution;
        }
        Ok(response)
    }
    /// Run GetSecret after validating the tenant and durably recording the command.
    async fn run_get_secret(
        &self,
        req: secrets::GetSecretRequest,
    ) -> Result<secrets::GetSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetSecret", "running workflow");
        let outgoing_0 = keymanagement::UpdateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
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
                funding: Some(keymanagement::crypto_key::Funding::ExternalAccountId(
                    req.secret_id.clone(),
                )),
                labels: (req.context.clone().unwrap_or_default().baggage.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyServiceClient::new(channel);
            let input: tonic::Request<keymanagement::UpdateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::GetSecretResponse::default();
        let contribution: Option<common::Money> = reply_0.crypto_key.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.secret.get_or_insert_with(Default::default).balance = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.secret_id.clone();
        if contribution != <String>::default() {
            response.secret.get_or_insert_with(Default::default).secret_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateSecret after validating the tenant and durably recording the command.
    async fn run_update_secret(
        &self,
        req: secrets::UpdateSecretRequest,
    ) -> Result<secrets::UpdateSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ScoreAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_by: req
                        .secret
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
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.update_secret_0_port)(outgoing_0).await?;
        let jobs_1 = req
            .secret
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = ratelimiting::RecalculateQuotaBucketsRequest {
                        context: req.context.clone(),
                        quota_bucket: Some(ratelimiting::QuotaBucket {
                            limits: vec![ratelimiting::QuotaBucketLimit {
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
                        let mut client = QuotaBucketCalculationServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::RecalculateQuotaBucketsRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.recalculate_quota_buckets(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = secrets::UpdateSecretResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.secret.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.secret.get_or_insert_with(Default::default).opened_at = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.secret.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.secret.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run DeleteSecret after validating the tenant and durably recording the command.
    async fn run_delete_secret(
        &self,
        req: secrets::DeleteSecretRequest,
    ) -> Result<secrets::DeleteSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteSecret", "running workflow");
        tracing::debug!(audit_value = ?req.reason.clone(), "recorded audit field");
        let outgoing_0 = secrets::LookupSecretRequest {
            context: req.context.clone(),
            secret_id: req.secret_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = SecretQueryServiceClient::new(channel);
            let input: tonic::Request<secrets::LookupSecretRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lookup_secret(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = featureflags::FindFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::find_flag_rollouts_request::Filter {
                created_after: reply_0
                    .secret
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: (reply_0.secret.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req.secret_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagRolloutQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::FindFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_flag_rollouts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::DeleteSecretResponse::default();
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ListSecrets after validating the tenant and durably recording the command.
    async fn run_list_secrets(
        &self,
        req: secrets::ListSecretsRequest,
    ) -> Result<secrets::ListSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = secrets::BrowseSecretVersionsRequest {
            context: req.context.clone(),
            filter: Some(secrets::browse_secret_versions_request::Filter {
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            page: Some(common::PageRequest {
                page_token: req.page.clone().unwrap_or_default().page_token.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.list_secrets_0_port.as_ref(), outgoing_0).await?;
        let mut response = secrets::ListSecretsResponse::default();
        let contribution: String = reply_0.page_info.clone().unwrap_or_default().next_page_token.clone();
        if contribution != <String>::default() {
            response.page_info.get_or_insert_with(Default::default).next_page_token = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run SearchSecrets after validating the tenant and durably recording the command.
    async fn run_search_secrets(
        &self,
        req: secrets::SearchSecretsRequest,
    ) -> Result<secrets::SearchSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SearchSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SearchSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CheckFeatureFlagEligibilityRequest {
            context: req.context.clone(),
            details: Some(featureflags::check_feature_flag_eligibility_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                owner: Some(identity::UserRef {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.search_secrets_0_port.as_ref(), outgoing_0).await?;
        let mut response = secrets::SearchSecretsResponse::default();
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.secrets.is_empty() {
                response.secrets.push(Default::default());
            }
            response.secrets[0].audit.get_or_insert_with(Default::default).etag = contribution;
        }
        Ok(response)
    }
    /// Run WatchSecrets after validating the tenant and durably recording the command.
    async fn run_watch_secrets(
        &self,
        req: secrets::WatchSecretsRequest,
    ) -> Result<secrets::WatchSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchSecrets", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = secrets::QuoteAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                action: Some(secrets::access_grant::Action::Block(secrets::AccessGrantBlockAction {
                    notify_email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(secrets::quote_access_grant_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.watch_secrets_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = secrets::WatchSecretsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.filter.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        let contribution: String = req.subscriber.clone().unwrap_or_default().phone.clone();
        if contribution != <String>::default() {
            response
                .secret
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .phone = contribution;
        }
        Ok(response)
    }
    /// Run BatchCreateSecrets after validating the tenant and durably recording the command.
    async fn run_batch_create_secrets(
        &self,
        req: secrets::BatchCreateSecretsRequest,
    ) -> Result<secrets::BatchCreateSecretsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchCreateSecrets", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateSecrets", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(secrets::batch_create_secrets_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = certificates::ScoreCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
                    display_name: (match req.chunk.clone() {
                        Some(secrets::batch_create_secrets_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .display_name
                    .clone(),
                    user_id: (match req.chunk.clone() {
                        Some(secrets::batch_create_secrets_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .holder
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                    ..Default::default()
                }),
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
                    email: (match req.chunk.clone() {
                        Some(secrets::batch_create_secrets_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                    ..Default::default()
                })),
                requester: Some(common::ContactInfo {
                    phone: (match req.chunk.clone() {
                        Some(secrets::batch_create_secrets_request::Chunk::Header(value)) => value,
                        _ => Default::default(),
                    })
                    .submitted_by
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
            let mut client = CsrValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ScoreCsrRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ExplainConfigChangeCalculationRequest {
            context: req.context.clone(),
            details: Some(configuration::explain_config_change_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    phone: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
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
        let reply_1 = acme_platform::ports::execute(self.batch_create_secrets_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = certificates::ExplainCsrDecisionRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
                    display_name: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .display_name
                    .clone(),
                    ..Default::default()
                }),
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
                    user_id: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                })),
                requester: Some(common::ContactInfo {
                    email: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .email
                    .clone(),
                    ..Default::default()
                }),
                sla: Some(certificates::CsrSla {
                    policy_id: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .sla
                        .clone()
                        .unwrap_or_default()
                        .policy_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CsrValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ExplainCsrDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_csr_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::BatchCreateSecretsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run CloneSecret after validating the tenant and durably recording the command.
    async fn run_clone_secret(
        &self,
        req: secrets::CloneSecretRequest,
    ) -> Result<secrets::CloneSecretResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneSecret", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneSecret", "running workflow");
        tracing::debug!(audit_value = ?req.secret.clone().unwrap_or_default().secret_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::SimulateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    updated_by: req
                        .secret
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
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.clone_secret_0_port)(outgoing_0).await?;
        let jobs_1 = req
            .secret
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
                    let outgoing_1 = ratelimiting::ValidateRateLimitRequest {
                        context: req.context.clone(),
                        rate_limit: Some(ratelimiting::RateLimit {
                            audit: Some(common::AuditInfo {
                                created_at: req
                                    .secret
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_at
                                    .clone(),
                                ..Default::default()
                            }),
                            action: Some(ratelimiting::rate_limit::Action::Discount(
                                ratelimiting::RateLimitDiscountAction {
                                    amount: item.amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            name: item.name.clone(),
                            owner: Some(identity::UserRef {
                                user_id: reply_0
                                    .access_grant
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
                    async move {
                        let mut client = RateLimitValidationServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::ValidateRateLimitRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.validate_rate_limit(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = authorization::RepairPermissionCheckRequest {
            context: req.context.clone(),
            details: Some(authorization::repair_permission_check_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            permission_check: Some(authorization::PermissionCheck {
                subject: Some(authorization::AccessPolicyRef {
                    display_name: req
                        .secret
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    version: reply_0.access_grant.clone().unwrap_or_default().version.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = PermissionCheckAdminServiceClient::new(channel);
            let input: tonic::Request<authorization::RepairPermissionCheckRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.repair_permission_check(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::ExportFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::export_flag_rollouts_request::Filter {
                created_after: reply_1
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.secret.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_1
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
        };
        let channel = self.channel.clone();
        let _reply_3 = async move {
            let mut client = FlagRolloutQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExportFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_flag_rollouts(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::ExportFlagRolloutsResponse::default();
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
        let mut response = secrets::CloneSecretResponse::default();
        let contribution: String = reply_0.access_grant.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response.secret.get_or_insert_with(Default::default).limits.is_empty() {
                response
                    .secret
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.secret.get_or_insert_with(Default::default).limits[0].name = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<identity::UserRef> = req.secret.clone().unwrap_or_default().holder.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.secret.get_or_insert_with(Default::default).holder = contribution;
        }
        let contribution: i32 = req.secret.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.secret.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CreateSecret for ExplainEnvironmentDecision.
struct CreateSecretDispatch0 {
    context: Option<common::RequestContext>,
    environment_audit_created_at: Option<prost_types::Timestamp>,
    environment_audit_updated_at: Option<prost_types::Timestamp>,
}
impl From<secrets::CreateSecretRequest> for CreateSecretDispatch0 {
    fn from(req: secrets::CreateSecretRequest) -> Self {
        Self {
            context: req.context.clone(),
            environment_audit_created_at: req
                .secret
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
            environment_audit_updated_at: req
                .secret
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
        }
    }
}
impl From<CreateSecretDispatch0> for configuration::ExplainEnvironmentDecisionRequest {
    fn from(value: CreateSecretDispatch0) -> Self {
        configuration::ExplainEnvironmentDecisionRequest {
            context: value.context,
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    created_at: value.environment_audit_created_at,
                    updated_at: value.environment_audit_updated_at,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl SecretService for SecretEndpoint {
    /// Creates a secret.
    async fn create_secret(
        &self,
        request: tonic::Request<secrets::CreateSecretRequest>,
    ) -> Result<tonic::Response<secrets::CreateSecretResponse>, tonic::Status> {
        self.run_create_secret(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a secret.
    async fn get_secret(
        &self,
        request: tonic::Request<secrets::GetSecretRequest>,
    ) -> Result<tonic::Response<secrets::GetSecretResponse>, tonic::Status> {
        self.run_get_secret(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Changes a secret.
    async fn update_secret(
        &self,
        request: tonic::Request<secrets::UpdateSecretRequest>,
    ) -> Result<tonic::Response<secrets::UpdateSecretResponse>, tonic::Status> {
        self.run_update_secret(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a secret.
    async fn delete_secret(
        &self,
        request: tonic::Request<secrets::DeleteSecretRequest>,
    ) -> Result<tonic::Response<secrets::DeleteSecretResponse>, tonic::Status> {
        self.run_delete_secret(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secrets a page at a time.
    async fn list_secrets(
        &self,
        request: tonic::Request<secrets::ListSecretsRequest>,
    ) -> Result<tonic::Response<secrets::ListSecretsResponse>, tonic::Status> {
        self.run_list_secrets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secrets a page at a time.
    async fn search_secrets(
        &self,
        request: tonic::Request<secrets::SearchSecretsRequest>,
    ) -> Result<tonic::Response<secrets::SearchSecretsResponse>, tonic::Status> {
        self.run_search_secrets(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type WatchSecretsStream =
        tokio_stream::wrappers::ReceiverStream<Result<secrets::WatchSecretsResponse, tonic::Status>>;
    /// Streams the changes of secrets as they happen.
    async fn watch_secrets(
        &self,
        request: tonic::Request<secrets::WatchSecretsRequest>,
    ) -> Result<tonic::Response<Self::WatchSecretsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_watch_secrets(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of secrets and answers once at the end.
    async fn batch_create_secrets(
        &self,
        request: tonic::Request<tonic::Streaming<secrets::BatchCreateSecretsRequest>>,
    ) -> Result<tonic::Response<secrets::BatchCreateSecretsResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = secrets::BatchCreateSecretsResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_secrets(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
    /// Creates a secret.
    async fn clone_secret(
        &self,
        request: tonic::Request<secrets::CloneSecretRequest>,
    ) -> Result<tonic::Response<secrets::CloneSecretResponse>, tonic::Status> {
        self.run_clone_secret(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
