use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::audit::v1::audit_event_validation_service_client::AuditEventValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_query_service_client::DocumentQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_service_client::LedgerAccountServiceClient;
use acme_proto::acme::ledger::v1::ledger_account_validation_service_client::LedgerAccountValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_calculation_service_server::RateLimitCalculationService;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::sponsorships::v1::sponsor_calculation_service_client::SponsorCalculationServiceClient;
use acme_proto::acme::telehealth::v1 as telehealth;
use acme_proto::acme::telehealth::v1::video_visit_query_service_client::VideoVisitQueryServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Computes the figures of a rate limit, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct RateLimitCalculationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
}
impl RateLimitCalculationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run CalculateRateLimit after validating the tenant and durably recording the command.
    async fn run_calculate_rate_limit(
        &self,
        req: ratelimiting::CalculateRateLimitRequest,
    ) -> Result<ratelimiting::CalculateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = configuration::ResolveConfigChangesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = venues::CountVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::count_venues_request::Filter {
                created_after: req
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                owner_id: req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeQueryServiceClient::new(channel_0);
                let input: tonic::Request<configuration::ResolveConfigChangesRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_config_changes(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = VenueQueryServiceClient::new(channel_1);
                let input: tonic::Request<venues::CountVenuesRequest> = tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_venues(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = configuration::ValidateEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = EnvironmentValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateEnvironmentRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_environment(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = venues::SuggestVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::suggest_venues_request::Filter {
                created_after: reply_2.outcome.clone().unwrap_or_default().effective_at.clone(),
                labels: reply_2.environment.clone().unwrap_or_default().specs.clone(),
                owner_id: req
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = VenueQueryServiceClient::new(channel);
            let input: tonic::Request<venues::SuggestVenuesRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.suggest_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CalculateRateLimitResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i32 = req.rate_limit.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run EstimateRateLimit after validating the tenant and durably recording the command.
    async fn run_estimate_rate_limit(
        &self,
        req: ratelimiting::EstimateRateLimitRequest,
    ) -> Result<ratelimiting::EstimateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = ledger::LintLedgerAccountsRequest {
            context: req.context.clone(),
            ledger_account: Some(ledger::LedgerAccount {
                audit: Some(common::AuditInfo {
                    etag: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::UpdateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            country_code: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            region: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .region
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = LedgerAccountValidationServiceClient::new(channel_0);
                let input: tonic::Request<ledger::LintLedgerAccountsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.lint_ledger_accounts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ConfigSetServiceClient::new(channel_1);
                let input: tonic::Request<configuration::UpdateConfigSetRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.update_config_set(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = configuration::EstimateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: reply_0
                                .ledger_account
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            lines: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .lines
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
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
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::EstimateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::EstimateRateLimitResponse::default();
        let contribution: String = reply_0
            .ledger_account
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
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run QuoteRateLimit after validating the tenant and durably recording the command.
    async fn run_quote_rate_limit(
        &self,
        req: ratelimiting::QuoteRateLimitRequest,
    ) -> Result<ratelimiting::QuoteRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = audit::BatchGetAuditEventsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::BatchGetAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::QuoteRateLimitResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
    /// Run CompareRateLimitOptions after validating the tenant and durably recording the command.
    async fn run_compare_rate_limit_options(
        &self,
        req: ratelimiting::CompareRateLimitOptionsRequest,
    ) -> Result<ratelimiting::CompareRateLimitOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareRateLimitOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareRateLimitOptions", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = venues::BrowseVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::browse_venues_request::Filter {
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = telehealth::CountVideoVisitsRequest {
            context: req.context.clone(),
            filter: Some(telehealth::count_video_visits_request::Filter {
                created_after: req
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: req
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .destination
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = VenueQueryServiceClient::new(channel_0);
                let input: tonic::Request<venues::BrowseVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.browse_venues(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = VideoVisitQueryServiceClient::new(channel_1);
                let input: tonic::Request<telehealth::CountVideoVisitsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_video_visits(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = ledger::CloneLedgerAccountRequest {
            context: req.context.clone(),
            ledger_account: Some(ledger::LedgerAccount {
                audit: Some(common::AuditInfo {
                    etag: req
                        .rate_limit
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
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
                limits: vec![ledger::LedgerAccountLimit {
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = LedgerAccountServiceClient::new(channel);
            let input: tonic::Request<ledger::CloneLedgerAccountRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_ledger_account(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::CheckConfigChangeEligibilityRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: reply_2
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
            details: Some(configuration::check_config_change_eligibility_request::Details {
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
                    name: reply_2
                        .ledger_account
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::CheckConfigChangeEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_config_change_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::CompareRateLimitOptionsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
            .ledger_account
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
        let contribution: String = reply_2
            .ledger_account
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_3.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i32 = req.rate_limit.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateRateLimits after validating the tenant and durably recording the command.
    async fn run_recalculate_rate_limits(
        &self,
        req: ratelimiting::RecalculateRateLimitsRequest,
    ) -> Result<ratelimiting::RecalculateRateLimitsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecalculateRateLimits", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateRateLimits", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = documents::ResolveDocumentsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::CreateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            region: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .region
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = DocumentQueryServiceClient::new(channel_0);
                let input: tonic::Request<documents::ResolveDocumentsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_documents(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ConfigSetServiceClient::new(channel_1);
                let input: tonic::Request<configuration::CreateConfigSetRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.create_config_set(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = ratelimiting::RecalculateRateLimitsResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_1.config_set.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i32 = req.rate_limit.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run SimulateRateLimit after validating the tenant and durably recording the command.
    async fn run_simulate_rate_limit(
        &self,
        req: ratelimiting::SimulateRateLimitRequest,
    ) -> Result<ratelimiting::SimulateRateLimitResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateRateLimit", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateRateLimit", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PreviewConfigChangeRequest {
            annotations: req.annotations.clone(),
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
            details: Some(configuration::preview_config_change_request::Details {
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::PreviewConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::SimulateRateLimitResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ExplainRateLimitCalculation after validating the tenant and durably recording the command.
    async fn run_explain_rate_limit_calculation(
        &self,
        req: ratelimiting::ExplainRateLimitCalculationRequest,
    ) -> Result<ratelimiting::ExplainRateLimitCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainRateLimitCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainRateLimitCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.rate_limit.clone().unwrap_or_default().rate_limit_id.clone(), "recorded audit field");
        let outgoing_0 = audit::NormalizeAuditEventRequest {
            audit_event: Some(audit::AuditEvent {
                subject: Some(audit::AuditTrailRef {
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
            details: Some(audit::normalize_audit_event_request::Details {
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
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            reason: (match req.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = sponsorships::RecalculateSponsorsRequest {
            context: req.context.clone(),
            sponsor: Some(sponsorships::Sponsor {
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
                relationship_manager: Some(identity::UserRef {
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = AuditEventValidationServiceClient::new(channel_0);
                let input: tonic::Request<audit::NormalizeAuditEventRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.normalize_audit_event(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = SponsorCalculationServiceClient::new(channel_1);
                let input: tonic::Request<sponsorships::RecalculateSponsorsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.recalculate_sponsors(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = ledger::CloneLedgerAccountRequest {
            context: req.context.clone(),
            ledger_account: Some(ledger::LedgerAccount {
                audit: Some(common::AuditInfo {
                    created_by: reply_1
                        .sponsor
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_by: reply_1
                        .sponsor
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    name: reply_1
                        .sponsor
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = LedgerAccountServiceClient::new(channel);
            let input: tonic::Request<ledger::CloneLedgerAccountRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_ledger_account(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = audit::UpdateAuditEventRequest {
            audit_event: Some(audit::AuditEvent {
                audit_event_id: reply_0.audit_event.clone().unwrap_or_default().audit_event_id.clone(),
                sequence: reply_0.audit_event.clone().unwrap_or_default().sequence.clone(),
                subject: Some(audit::AuditTrailRef {
                    display_name: reply_0
                        .audit_event
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    number: (match reply_1.sponsor.clone().unwrap_or_default().identity_document.clone() {
                        Some(sponsorships::sponsor::IdentityDocument::Passport(value)) => value,
                        _ => Default::default(),
                    })
                    .number
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
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::UpdateAuditEventRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_audit_event(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::ExplainRateLimitCalculationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .sponsor
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .rate_limit
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1
            .sponsor
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        let contribution: i32 = req.rate_limit.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.rate_limit.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run GetRateLimitRates after validating the tenant and durably recording the command.
    async fn run_get_rate_limit_rates(
        &self,
        req: ratelimiting::GetRateLimitRatesRequest,
    ) -> Result<ratelimiting::GetRateLimitRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetRateLimitRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetRateLimitRates", "running workflow");
        let outgoing_0 = featureflags::RecalculateFlagEvaluationsRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                attributes: req.context.clone().unwrap_or_default().baggage.clone(),
                correlation_id: req.rate_limit_id.clone(),
                source: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                subject: Some(featureflags::FlagRolloutRef {
                    starts_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagEvaluationCalculationServiceClient::new(channel);
            let input: tonic::Request<featureflags::RecalculateFlagEvaluationsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_flag_evaluations(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::GetRateLimitRatesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.rate_limit_id.clone();
        if contribution != <String>::default() {
            response.rate_limit.get_or_insert_with(Default::default).rate_limit_id = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { RateLimitCalculationService, RateLimitCalculationController;
unary {
calculate_rate_limit (ratelimiting::CalculateRateLimitRequest) -> ratelimiting::CalculateRateLimitResponse => run_calculate_rate_limit;
estimate_rate_limit (ratelimiting::EstimateRateLimitRequest) -> ratelimiting::EstimateRateLimitResponse => run_estimate_rate_limit;
quote_rate_limit (ratelimiting::QuoteRateLimitRequest) -> ratelimiting::QuoteRateLimitResponse => run_quote_rate_limit;
compare_rate_limit_options (ratelimiting::CompareRateLimitOptionsRequest) -> ratelimiting::CompareRateLimitOptionsResponse => run_compare_rate_limit_options;
recalculate_rate_limits (ratelimiting::RecalculateRateLimitsRequest) -> ratelimiting::RecalculateRateLimitsResponse => run_recalculate_rate_limits;
simulate_rate_limit (ratelimiting::SimulateRateLimitRequest) -> ratelimiting::SimulateRateLimitResponse => run_simulate_rate_limit;
explain_rate_limit_calculation (ratelimiting::ExplainRateLimitCalculationRequest) -> ratelimiting::ExplainRateLimitCalculationResponse => run_explain_rate_limit_calculation;
get_rate_limit_rates (ratelimiting::GetRateLimitRatesRequest) -> ratelimiting::GetRateLimitRatesResponse => run_get_rate_limit_rates;
}
server {
}
client {
}
bidi {
}
}
