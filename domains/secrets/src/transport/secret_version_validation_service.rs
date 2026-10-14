use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::secret_version_validation_service_server::SecretVersionValidationService;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::shows::v1::show_calculation_service_client::ShowCalculationServiceClient;

/// Checks secret versions against the rules before anything acts on them.
#[derive(Clone)]
pub struct SecretVersionValidationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
}
impl SecretVersionValidationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run ValidateSecretVersion after validating the tenant and durably recording the command.
    async fn run_validate_secret_version(
        &self,
        req: secrets::ValidateSecretVersionRequest,
    ) -> Result<secrets::ValidateSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateSecretVersion", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::RevertFlagRolloutRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        phone: req
                            .details
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::RevertFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.revert_flag_rollout(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ValidateSecretVersionResponse::default();
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
        let contribution: String = req.secret_version.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run PreviewSecretVersion after validating the tenant and durably recording the command.
    async fn run_preview_secret_version(
        &self,
        req: secrets::PreviewSecretVersionRequest,
    ) -> Result<secrets::PreviewSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewSecretVersion", "running workflow");
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
                    let outgoing_0 = secrets::EstimateAccessGrantRequest {
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
                            audit: Some(common::AuditInfo {
                                updated_at: item.observed_at.clone(),
                                ..Default::default()
                            }),
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
                            version: req
                                .secret_version
                                .clone()
                                .unwrap_or_default()
                                .subject
                                .clone()
                                .unwrap_or_default()
                                .version
                                .clone(),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessGrantCalculationServiceClient::new(channel);
                        let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.estimate_access_grant(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = secrets::PreviewSecretVersionResponse::default();
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
    /// Run CheckSecretVersionEligibility after validating the tenant and durably recording the command.
    async fn run_check_secret_version_eligibility(
        &self,
        req: secrets::CheckSecretVersionEligibilityRequest,
    ) -> Result<secrets::CheckSecretVersionEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckSecretVersionEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckSecretVersionEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::CalculateAccessGrantRequest {
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
                    email: req
                        .details
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
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::CalculateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .secret_version
            .clone()
            .unwrap_or_default()
            .measurements
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_1 = secrets::SimulateAccessGrantRequest {
                        access_grant: Some(secrets::AccessGrant {
                            audit: Some(common::AuditInfo {
                                created_at: item.observed_at.clone(),
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
                            action: Some(secrets::access_grant::Action::Route(secrets::AccessGrantRouteAction {
                                weight: (match reply_0.access_grant.clone().unwrap_or_default().action.clone() {
                                    Some(secrets::access_grant::Action::Route(value)) => value,
                                    _ => Default::default(),
                                })
                                .weight
                                .clone(),
                                ..Default::default()
                            })),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessGrantCalculationServiceClient::new(channel);
                        let input: tonic::Request<secrets::SimulateAccessGrantRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.simulate_access_grant(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = secrets::CheckSecretVersionEligibilityResponse::default();
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
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
    /// Run NormalizeSecretVersion after validating the tenant and durably recording the command.
    async fn run_normalize_secret_version(
        &self,
        req: secrets::NormalizeSecretVersionRequest,
    ) -> Result<secrets::NormalizeSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeSecretVersion", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::GetAccessGrantRatesRequest {
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
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::GetAccessGrantRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_access_grant_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::NormalizeSecretVersionResponse::default();
        let contribution: i64 = reply_0.access_grant.clone().unwrap_or_default().version.clone();
        if contribution != <i64>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .version = contribution;
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
    /// Run ExplainSecretVersionDecision after validating the tenant and durably recording the command.
    async fn run_explain_secret_version_decision(
        &self,
        req: secrets::ExplainSecretVersionDecisionRequest,
    ) -> Result<secrets::ExplainSecretVersionDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainSecretVersionDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainSecretVersionDecision", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = shows::CompareShowOptionsRequest {
            context: req.context.clone(),
            show: Some(shows::Show {
                organizer: Some(identity::UserRef {
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
                participants: vec![shows::ShowParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        phone: req
                            .details
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ShowCalculationServiceClient::new(channel);
            let input: tonic::Request<shows::CompareShowOptionsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_show_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::GetAccessGrantRatesRequest {
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
        let reply_1 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::GetAccessGrantRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_access_grant_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ExplainSecretVersionDecisionResponse::default();
        let contribution: String = reply_0
            .show
            .clone()
            .unwrap_or_default()
            .organizer
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
        let contribution: String = req.secret_version.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run ScoreSecretVersion after validating the tenant and durably recording the command.
    async fn run_score_secret_version(
        &self,
        req: secrets::ScoreSecretVersionRequest,
    ) -> Result<secrets::ScoreSecretVersionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreSecretVersion", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreSecretVersion", "running workflow");
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
                    let outgoing_0 = secrets::EstimateAccessGrantRequest {
                        access_grant: Some(secrets::AccessGrant {
                            audit: Some(common::AuditInfo {
                                created_at: item.observed_at.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        details: Some(secrets::estimate_access_grant_request::Details {
                            destination: Some(common::PostalAddress {
                                postal_code: req
                                    .details
                                    .clone()
                                    .unwrap_or_default()
                                    .destination
                                    .clone()
                                    .unwrap_or_default()
                                    .postal_code
                                    .clone(),
                                recipient: req
                                    .details
                                    .clone()
                                    .unwrap_or_default()
                                    .destination
                                    .clone()
                                    .unwrap_or_default()
                                    .recipient
                                    .clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AccessGrantCalculationServiceClient::new(channel);
                        let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.estimate_access_grant(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = secrets::QuoteAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                action: Some(secrets::access_grant::Action::Route(secrets::AccessGrantRouteAction {
                    weight: (match reply_0.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Route(value)) => value,
                        _ => Default::default(),
                    })
                    .weight
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::QuoteAccessGrantRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::ScoreSecretVersionResponse::default();
        let contribution: Option<common::Money> = reply_1.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
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
    /// Run LintSecretVersions after validating the tenant and durably recording the command.
    async fn run_lint_secret_versions(
        &self,
        req: secrets::LintSecretVersionsRequest,
    ) -> Result<secrets::LintSecretVersionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintSecretVersions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintSecretVersions", "running workflow");
        tracing::debug!(audit_value = ?req.secret_version.clone().unwrap_or_default().secret_version_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::CompareFlagRolloutOptionsRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        name: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        phone: req
                            .details
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::CompareFlagRolloutOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_flag_rollout_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_by: reply_0
                        .flag_rollout
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = secrets::CalculateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_at: reply_0
                        .flag_rollout
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
            }),
            context: req.context.clone(),
            details: Some(secrets::calculate_access_grant_request::Details {
                destination: Some(common::PostalAddress {
                    locality: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::CalculateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = secrets::SimulateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                owner: Some(identity::UserRef {
                    display_name: reply_0
                        .flag_rollout
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let reply_3 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::SimulateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::LintSecretVersionsResponse::default();
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .organizer
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
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.access_grant.clone().unwrap_or_default().access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
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
    /// Run GetSecretVersionRules after validating the tenant and durably recording the command.
    async fn run_get_secret_version_rules(
        &self,
        req: secrets::GetSecretVersionRulesRequest,
    ) -> Result<secrets::GetSecretVersionRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetSecretVersionRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetSecretVersionRules", "running workflow");
        let outgoing_0 = secrets::QuoteAccessGrantRequest {
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::QuoteAccessGrantRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::GetSecretVersionRulesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.access_grant.clone().unwrap_or_default().access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .secret_version
                .get_or_insert_with(Default::default)
                .subject
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
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
acme_platform::endpoints! { SecretVersionValidationService, SecretVersionValidationApi;
unary {
validate_secret_version (secrets::ValidateSecretVersionRequest) -> secrets::ValidateSecretVersionResponse => run_validate_secret_version;
preview_secret_version (secrets::PreviewSecretVersionRequest) -> secrets::PreviewSecretVersionResponse => run_preview_secret_version;
check_secret_version_eligibility (secrets::CheckSecretVersionEligibilityRequest) -> secrets::CheckSecretVersionEligibilityResponse => run_check_secret_version_eligibility;
normalize_secret_version (secrets::NormalizeSecretVersionRequest) -> secrets::NormalizeSecretVersionResponse => run_normalize_secret_version;
explain_secret_version_decision (secrets::ExplainSecretVersionDecisionRequest) -> secrets::ExplainSecretVersionDecisionResponse => run_explain_secret_version_decision;
score_secret_version (secrets::ScoreSecretVersionRequest) -> secrets::ScoreSecretVersionResponse => run_score_secret_version;
lint_secret_versions (secrets::LintSecretVersionsRequest) -> secrets::LintSecretVersionsResponse => run_lint_secret_versions;
get_secret_version_rules (secrets::GetSecretVersionRulesRequest) -> secrets::GetSecretVersionRulesResponse => run_get_secret_version_rules;
}
server {
}
client {
}
bidi {
}
}
