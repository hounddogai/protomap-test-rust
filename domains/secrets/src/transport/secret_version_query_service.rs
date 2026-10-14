use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::careplans::v1::care_plan_query_service_client::CarePlanQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_history_service_client::FeatureFlagHistoryServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_client::FlagRolloutQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::secret_version_query_service_server::SecretVersionQueryService;
use acme_proto::acme::secrets::v1::secret_version_validation_service_client::SecretVersionValidationServiceClient;

/// Answers read-only questions about secret versions, for screens and reports.
#[derive(Clone)]
pub struct SecretVersionQueryController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    find_secret_versions_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<secrets::PreviewSecretVersionRequest, secrets::PreviewSecretVersionResponse>,
    >,
    peek_secret_version_0_port: acme_platform::ports::Callback<
        secrets::CompareAccessGrantOptionsRequest,
        secrets::CompareAccessGrantOptionsResponse,
    >,
    peek_secret_version_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<secrets::EstimateAccessGrantRequest, secrets::EstimateAccessGrantResponse>,
    >,
    suggest_secret_versions_0_port:
        acme_platform::ports::Callback<certificates::CompareCsrOptionsRequest, certificates::CompareCsrOptionsResponse>,
}
impl SecretVersionQueryController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            secrets::PreviewSecretVersionRequest,
            secrets::PreviewSecretVersionResponse,
        > = {
            let client = SecretVersionValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::PreviewSecretVersionRequest| -> acme_platform::ports::RpcFuture<secrets::PreviewSecretVersionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_secret_version(request).await?.into_inner())
})
})
        };
        let find_secret_versions_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::CompareCsrOptionsRequest,
            certificates::CompareCsrOptionsResponse,
        > = {
            let client = CsrCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CompareCsrOptionsRequest| -> acme_platform::ports::RpcFuture<certificates::CompareCsrOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_csr_options(request).await?.into_inner())
})
})
        };
        let suggest_secret_versions_0_port = callback;
        let callback: acme_platform::ports::Callback<
            secrets::CompareAccessGrantOptionsRequest,
            secrets::CompareAccessGrantOptionsResponse,
        > = {
            let client = AccessGrantCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::CompareAccessGrantOptionsRequest| -> acme_platform::ports::RpcFuture<secrets::CompareAccessGrantOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_access_grant_options(request).await?.into_inner())
})
})
        };
        let peek_secret_version_0_port = callback;
        let callback: acme_platform::ports::Callback<
            secrets::EstimateAccessGrantRequest,
            secrets::EstimateAccessGrantResponse,
        > = {
            let client = AccessGrantCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: secrets::EstimateAccessGrantRequest| -> acme_platform::ports::RpcFuture<secrets::EstimateAccessGrantResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_access_grant(request).await?.into_inner())
})
})
        };
        let peek_secret_version_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            find_secret_versions_0_port,
            peek_secret_version_0_port,
            peek_secret_version_1_port,
            suggest_secret_versions_0_port,
        }
    }
    /// Run LookupSecretVersion after validating the tenant and durably recording the command.
    async fn run_lookup_secret_version(
        &self,
        req: secrets::LookupSecretVersionRequest,
    ) -> Result<secrets::LookupSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupSecretVersion", "running workflow");
        let outgoing_0 = careplans::ExportCarePlansRequest {
            context: req.context.clone(),
            filter: Some(careplans::export_care_plans_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.secret_version_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = CarePlanQueryServiceClient::new(channel);
            let input: tonic::Request<careplans::ExportCarePlansRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_care_plans(input).await?;
            let mut frames = response.into_inner();
            let mut last = careplans::ExportCarePlansResponse::default();
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
        let mut response = secrets::LookupSecretVersionResponse::default();
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseSecretVersions after validating the tenant and durably recording the command.
    async fn run_browse_secret_versions(
        &self,
        req: secrets::BrowseSecretVersionsRequest,
    ) -> Result<secrets::BrowseSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseSecretVersions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let command_0 = BrowseSecretVersionsDispatch0::from(req.clone());
        let outgoing_0: secrets::SimulateAccessGrantRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::SimulateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::CompareAccessGrantOptionsRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
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
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::CompareAccessGrantOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_access_grant_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::BrowseSecretVersionsResponse::default();
        let contribution: i64 = reply_0.access_grant.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            if response.secret_versions.is_empty() {
                response.secret_versions.push(Default::default());
            }
            response.secret_versions[0]
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response.secret_versions.is_empty() {
                response.secret_versions.push(Default::default());
            }
            response.secret_versions[0]
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run FindSecretVersions after validating the tenant and durably recording the command.
    async fn run_find_secret_versions(
        &self,
        req: secrets::FindSecretVersionsRequest,
    ) -> Result<secrets::FindSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindSecretVersions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = secrets::PreviewSecretVersionRequest {
            context: req.context.clone(),
            details: Some(secrets::preview_secret_version_request::Details {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.find_secret_versions_0_port.as_ref(), outgoing_0).await?;
        let mut response = secrets::FindSecretVersionsResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run CountSecretVersions after validating the tenant and durably recording the command.
    async fn run_count_secret_versions(
        &self,
        req: secrets::CountSecretVersionsRequest,
    ) -> Result<secrets::CountSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountSecretVersions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                comments: vec![certificates::CsrComment {
                    created_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }],
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
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
                })),
                requester: Some(common::ContactInfo {
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
                ..Default::default()
            }),
            details: Some(certificates::estimate_csr_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<certificates::EstimateCsrRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CsrCalculationServiceClient::new(channel);
                let input: tonic::Request<certificates::EstimateCsrRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.estimate_csr(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_0)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_0 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let outgoing_1 = secrets::CalculateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
                        .csr
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
                    display_name: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .display_name
                    .clone(),
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
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::CalculateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = secrets::CountSecretVersionsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run ResolveSecretVersions after validating the tenant and durably recording the command.
    async fn run_resolve_secret_versions(
        &self,
        req: secrets::ResolveSecretVersionsRequest,
    ) -> Result<secrets::ResolveSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveSecretVersions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveSecretVersions", "running workflow");
        let response = secrets::ResolveSecretVersionsResponse::default();
        Ok(response)
    }
    /// Run ExportSecretVersions after validating the tenant and durably recording the command.
    async fn run_export_secret_versions(
        &self,
        req: secrets::ExportSecretVersionsRequest,
    ) -> Result<secrets::ExportSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportSecretVersions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ExportFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::export_flag_rollouts_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = FlagRolloutQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExportFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
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
        let outgoing_1 = secrets::ScoreSecretVersionRequest {
            context: req.context.clone(),
            details: Some(secrets::score_secret_version_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SecretVersionValidationServiceClient::new(channel);
            let input: tonic::Request<secrets::ScoreSecretVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_secret_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ExportSecretVersionsResponse::default();
        let contribution: i64 = reply_1.secret_version.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.sequence = contribution;
        }
        let contribution: String = reply_1
            .secret_version
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeSecretVersions after validating the tenant and durably recording the command.
    async fn run_summarize_secret_versions(
        &self,
        req: secrets::SummarizeSecretVersionsRequest,
    ) -> Result<secrets::SummarizeSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SummarizeSecretVersions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CalculateConfigChangeRequest {
            annotations: (req.annotations.clone())
                .into_iter()
                .filter(|(key, _)| !key.is_empty())
                .map(|(key, value)| (key, value.trim().to_owned()))
                .collect(),
            config_change: Some(configuration::ConfigChange {
                attributes: (req.details.clone().unwrap_or_default().filters.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                measurements: vec![configuration::ConfigChangeMeasurement {
                    observed_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CalculateConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::RecordFeatureFlagVersionRequest {
            context: req.context.clone(),
            details: Some(featureflags::record_feature_flag_version_request::Details {
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                action: Some(featureflags::feature_flag::Action::Block(
                    featureflags::FeatureFlagBlockAction {
                        reason: req.reason.clone(),
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    display_name: reply_0
                        .config_change
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
        let reply_1 = async move {
            let mut client = FeatureFlagHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::RecordFeatureFlagVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.record_feature_flag_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::SummarizeSecretVersionsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestSecretVersions after validating the tenant and durably recording the command.
    async fn run_suggest_secret_versions(
        &self,
        req: secrets::SuggestSecretVersionsRequest,
    ) -> Result<secrets::SuggestSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestSecretVersions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::CompareCsrOptionsRequest {
            context: req.context.clone(),
            details: Some(certificates::compare_csr_options_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.suggest_secret_versions_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = secrets::SuggestSecretVersionsResponse::default();
        let contribution: String = (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
            Some(certificates::csr::Origin::Customer(value)) => value,
            _ => Default::default(),
        })
        .display_name
        .clone();
        if contribution != <String>::default() {
            if response.secret_versions.is_empty() {
                response.secret_versions.push(Default::default());
            }
            response.secret_versions[0]
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run PeekSecretVersion after validating the tenant and durably recording the command.
    async fn run_peek_secret_version(
        &self,
        req: secrets::PeekSecretVersionRequest,
    ) -> Result<secrets::PeekSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekSecretVersion", "running workflow");
        let outgoing_0 = secrets::CompareAccessGrantOptionsRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
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
            ..Default::default()
        };
        let reply_0 = (self.peek_secret_version_0_port)(outgoing_0).await?;
        let outgoing_1 = secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: reply_0
                        .access_grant
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
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.peek_secret_version_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = secrets::PeekSecretVersionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .correlation_id = contribution;
        }
        let contribution: i64 = reply_0.access_grant.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: i64 = reply_1.access_grant.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
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
        let contribution: String = req.secret_version_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .secret_version_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by BrowseSecretVersions for SimulateAccessGrant.
struct BrowseSecretVersionsDispatch0 {
    context: Option<common::RequestContext>,
    details_destination_region: String,
    access_grant_owner_user_id: String,
    access_grant_owner_email: String,
}
impl From<secrets::BrowseSecretVersionsRequest> for BrowseSecretVersionsDispatch0 {
    fn from(req: secrets::BrowseSecretVersionsRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_destination_region: req.filter.clone().unwrap_or_default().region.clone(),
            access_grant_owner_user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            access_grant_owner_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
        }
    }
}
impl From<BrowseSecretVersionsDispatch0> for secrets::SimulateAccessGrantRequest {
    fn from(value: BrowseSecretVersionsDispatch0) -> Self {
        secrets::SimulateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                owner: Some(identity::UserRef {
                    email: value.access_grant_owner_email,
                    user_id: value.access_grant_owner_user_id,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            details: Some(secrets::simulate_access_grant_request::Details {
                destination: Some(common::PostalAddress {
                    region: value.details_destination_region,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl SecretVersionQueryService for SecretVersionQueryController {
    /// Returns a secret version.
    async fn lookup_secret_version(
        &self,
        request: tonic::Request<secrets::LookupSecretVersionRequest>,
    ) -> Result<tonic::Response<secrets::LookupSecretVersionResponse>, tonic::Status> {
        self.run_lookup_secret_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secret versions a page at a time.
    async fn browse_secret_versions(
        &self,
        request: tonic::Request<secrets::BrowseSecretVersionsRequest>,
    ) -> Result<tonic::Response<secrets::BrowseSecretVersionsResponse>, tonic::Status> {
        self.run_browse_secret_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secret versions a page at a time.
    async fn find_secret_versions(
        &self,
        request: tonic::Request<secrets::FindSecretVersionsRequest>,
    ) -> Result<tonic::Response<secrets::FindSecretVersionsResponse>, tonic::Status> {
        self.run_find_secret_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts secret versions by status.
    async fn count_secret_versions(
        &self,
        request: tonic::Request<secrets::CountSecretVersionsRequest>,
    ) -> Result<tonic::Response<secrets::CountSecretVersionsResponse>, tonic::Status> {
        self.run_count_secret_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several secret versions at once.
    async fn resolve_secret_versions(
        &self,
        request: tonic::Request<secrets::ResolveSecretVersionsRequest>,
    ) -> Result<tonic::Response<secrets::ResolveSecretVersionsResponse>, tonic::Status> {
        self.run_resolve_secret_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportSecretVersionsStream =
        tokio_stream::wrappers::ReceiverStream<Result<secrets::ExportSecretVersionsResponse, tonic::Status>>;
    /// Streams every matching secret version.
    async fn export_secret_versions(
        &self,
        request: tonic::Request<secrets::ExportSecretVersionsRequest>,
    ) -> Result<tonic::Response<Self::ExportSecretVersionsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(async move { handler.run_export_secret_versions(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a secret version.
    async fn summarize_secret_versions(
        &self,
        request: tonic::Request<secrets::SummarizeSecretVersionsRequest>,
    ) -> Result<tonic::Response<secrets::SummarizeSecretVersionsResponse>, tonic::Status> {
        self.run_summarize_secret_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists secret versions a page at a time.
    async fn suggest_secret_versions(
        &self,
        request: tonic::Request<secrets::SuggestSecretVersionsRequest>,
    ) -> Result<tonic::Response<secrets::SuggestSecretVersionsResponse>, tonic::Status> {
        self.run_suggest_secret_versions(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a secret version.
    async fn peek_secret_version(
        &self,
        request: tonic::Request<secrets::PeekSecretVersionRequest>,
    ) -> Result<tonic::Response<secrets::PeekSecretVersionResponse>, tonic::Status> {
        self.run_peek_secret_version(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
