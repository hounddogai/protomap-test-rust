use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::environment_lifecycle_service_server::EnvironmentLifecycleService;
use acme_proto::acme::configuration::v1::environment_reporting_service_client::EnvironmentReportingServiceClient;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::dispatch::v1::dispatch_order_service_client::DispatchOrderServiceClient;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::documents::v1::document_service_client::DocumentServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_validation_service_client::KeyRotationValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::reviews::v1 as reviews;
use acme_proto::acme::reviews::v1::product_review_service_client::ProductReviewServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_service_client::SearchIndexServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Moves an environment between the states of its lifecycle.
#[derive(Clone)]
pub struct EnvironmentLifecycleController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    activate_environment_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<keymanagement::LintCryptoKeysRequest, keymanagement::LintCryptoKeysResponse>,
    >,
    activate_environment_2_port:
        acme_platform::ports::Callback<ratelimiting::CloneRateLimitRequest, ratelimiting::CloneRateLimitResponse>,
    activate_environment_3_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::GenerateConfigChangeReportRequest,
            configuration::GenerateConfigChangeReportResponse,
        >,
    >,
    schedule_environment_expiry_0_port:
        acme_platform::ports::Callback<search::CreateSearchIndexRequest, search::CreateSearchIndexResponse>,
    schedule_environment_expiry_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            configuration::ListConfigSetReportsRequest,
            configuration::ListConfigSetReportsResponse,
        >,
    >,
}
impl EnvironmentLifecycleController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::LintCryptoKeysRequest,
            keymanagement::LintCryptoKeysResponse,
        > = {
            let client = CryptoKeyValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::LintCryptoKeysRequest| -> acme_platform::ports::RpcFuture<keymanagement::LintCryptoKeysResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_crypto_keys(request).await?.into_inner())
})
})
        };
        let activate_environment_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let activate_environment_2_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::GenerateConfigChangeReportRequest,
            configuration::GenerateConfigChangeReportResponse,
        > = {
            let client = ConfigChangeReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::GenerateConfigChangeReportRequest| -> acme_platform::ports::RpcFuture<configuration::GenerateConfigChangeReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.generate_config_change_report(request).await?.into_inner())
})
})
        };
        let activate_environment_3_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            search::CreateSearchIndexRequest,
            search::CreateSearchIndexResponse,
        > = {
            let client = SearchIndexServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: search::CreateSearchIndexRequest| -> acme_platform::ports::RpcFuture<search::CreateSearchIndexResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.create_search_index(request).await?.into_inner())
})
})
        };
        let schedule_environment_expiry_0_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::ListConfigSetReportsRequest,
            configuration::ListConfigSetReportsResponse,
        > = {
            let client = ConfigSetReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ListConfigSetReportsRequest| -> acme_platform::ports::RpcFuture<configuration::ListConfigSetReportsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_config_set_reports(request).await?.into_inner())
})
})
        };
        let schedule_environment_expiry_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            activate_environment_0_port,
            activate_environment_2_port,
            activate_environment_3_port,
            schedule_environment_expiry_0_port,
            schedule_environment_expiry_1_port,
        }
    }
    /// Run ActivateEnvironment after validating the tenant and durably recording the command.
    async fn run_activate_environment(
        &self,
        req: configuration::ActivateEnvironmentRequest,
    ) -> Result<configuration::ActivateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ActivateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ActivateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::LintCryptoKeysRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
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
        let reply_0 = acme_platform::ports::execute(self.activate_environment_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = documents::WatchDocumentsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .new_owner
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                name: reply_0
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: reply_0
                    .crypto_key
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify_contact
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
            let mut client = DocumentServiceClient::new(channel);
            let input: tonic::Request<documents::WatchDocumentsRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_documents(input).await?;
            let mut frames = response.into_inner();
            let mut last = documents::WatchDocumentsResponse::default();
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
        let outgoing_2 = ratelimiting::CloneRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_by: reply_0
                        .crypto_key
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
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify_contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
        let reply_2 = (self.activate_environment_2_port)(outgoing_2).await?;
        let outgoing_3 = configuration::GenerateConfigChangeReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(configuration::generate_config_change_report_request::Details {
                filters: reply_0.crypto_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            reason: (match reply_2.rate_limit.clone().unwrap_or_default().action.clone() {
                Some(ratelimiting::rate_limit::Action::Block(value)) => value,
                _ => Default::default(),
            })
            .reason
            .clone(),
            ..Default::default()
        };
        let reply_3 =
            self.activate_environment_3_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_3)
            .await?;
        let outgoing_4 = reviews::BatchCreateProductReviewsRequest {
            context: req.context.clone(),
            chunk: Some(reviews::batch_create_product_reviews_request::Chunk::Record(
                reviews::ProductReview {
                    counterparty: Some(customers::CustomerRef {
                        display_name: reply_2
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
                    lines: vec![reviews::ProductReviewLine {
                        amount: (match reply_2.rate_limit.clone().unwrap_or_default().action.clone() {
                            Some(ratelimiting::rate_limit::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
                        .clone(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = ProductReviewServiceClient::new(channel);
            let input: tonic::Request<reviews::BatchCreateProductReviewsRequest> =
                tonic::IntoRequest::into_request(outgoing_4);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_product_reviews(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                    subject: Some(featureflags::FlagRolloutRef {
                        display_name: reply_0
                            .crypto_key
                            .clone()
                            .unwrap_or_default()
                            .holder
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        starts_at: reply_2
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_5 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_5);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_flag_evaluation_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagEvaluationChangesResponse::default();
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
        let mut response = configuration::ActivateEnvironmentResponse::default();
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run SuspendEnvironment after validating the tenant and durably recording the command.
    async fn run_suspend_environment(
        &self,
        req: configuration::SuspendEnvironmentRequest,
    ) -> Result<configuration::SuspendEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuspendEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuspendEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = audit::CountAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::count_audit_events_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: req.annotations.clone(),
                owner_id: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .new_owner
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            email: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .new_owner
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = AuditEventQueryServiceClient::new(channel_0);
                let input: tonic::Request<audit::CountAuditEventsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_audit_events(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ConfigSetCalculationServiceClient::new(channel_1);
                let input: tonic::Request<configuration::RecalculateConfigSetsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.recalculate_config_sets(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = configuration::SuspendEnvironmentResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ReinstateEnvironment after validating the tenant and durably recording the command.
    async fn run_reinstate_environment(
        &self,
        req: configuration::ReinstateEnvironmentRequest,
    ) -> Result<configuration::ReinstateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReinstateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReinstateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = venues::CloneVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .new_owner
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
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::CloneVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::CloneConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            region: (match reply_0.venue.clone().unwrap_or_default().locator.clone() {
                                Some(venues::venue::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::CloneConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ReinstateEnvironmentResponse::default();
        let contribution: String = reply_0.venue.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = reply_1
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run TerminateEnvironment after validating the tenant and durably recording the command.
    async fn run_terminate_environment(
        &self,
        req: configuration::TerminateEnvironmentRequest,
    ) -> Result<configuration::TerminateEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("TerminateEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "TerminateEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let command_0 = TerminateEnvironmentDispatch0::from(req.clone());
        let outgoing_0: keymanagement::NormalizeKeyRotationRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::NormalizeKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::TerminateEnvironmentResponse::default();
        let contribution: String = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run RenewEnvironment after validating the tenant and durably recording the command.
    async fn run_renew_environment(
        &self,
        req: configuration::RenewEnvironmentRequest,
    ) -> Result<configuration::RenewEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RenewEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RenewEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let outgoing_0 = configuration::ScheduleEnvironmentReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = documents::BatchGetDocumentsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = EnvironmentReportingServiceClient::new(channel_0);
                let input: tonic::Request<configuration::ScheduleEnvironmentReportRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.schedule_environment_report(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = DocumentServiceClient::new(channel_1);
                let input: tonic::Request<documents::BatchGetDocumentsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_documents(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = configuration::RenewEnvironmentResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run ExpireEnvironment after validating the tenant and durably recording the command.
    async fn run_expire_environment(
        &self,
        req: configuration::ExpireEnvironmentRequest,
    ) -> Result<configuration::ExpireEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExpireEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExpireEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().reason_code.clone(), "recorded audit field");
        let command_0 = ExpireEnvironmentDispatch0::from(req.clone());
        let outgoing_0: configuration::FindConfigChangesRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::FindConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::FindCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::find_crypto_keys_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: req.annotations.clone(),
                owner_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::codec::through_any(&outgoing_1, "acme.keymanagement.v1.FindCryptoKeysRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyQueryServiceClient::new(channel);
            let input: tonic::Request<keymanagement::FindCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = dispatch::SearchDispatchOrdersRequest {
            context: req.context.clone(),
            filter: Some(dispatch::search_dispatch_orders_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: reply_1.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<dispatch::SearchDispatchOrdersRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = DispatchOrderServiceClient::new(channel);
                let input: tonic::Request<dispatch::SearchDispatchOrdersRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.search_dispatch_orders(input).await?;
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
        let mut response = configuration::ExpireEnvironmentResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.details.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        Ok(response)
    }
    /// Run GetEnvironmentLifecycle after validating the tenant and durably recording the command.
    async fn run_get_environment_lifecycle(
        &self,
        req: configuration::GetEnvironmentLifecycleRequest,
    ) -> Result<configuration::GetEnvironmentLifecycleResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetEnvironmentLifecycle", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetEnvironmentLifecycle", "running workflow");
        let outgoing_0 = configuration::ScoreConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                attributes: (req.context.clone().unwrap_or_default().baggage.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                occurred_at: req.context.clone().unwrap_or_default().deadline.clone(),
                source: req.environment_id.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ScoreConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::GetEnvironmentLifecycleResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        Ok(response)
    }
    /// Run ScheduleEnvironmentExpiry after validating the tenant and durably recording the command.
    async fn run_schedule_environment_expiry(
        &self,
        req: configuration::ScheduleEnvironmentExpiryRequest,
    ) -> Result<configuration::ScheduleEnvironmentExpiryResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ScheduleEnvironmentExpiry", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScheduleEnvironmentExpiry", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = search::CreateSearchIndexRequest {
            context: req.context.clone(),
            search_index: Some(search::SearchIndex {
                dimensions: Some(search::SearchIndexDimensions {
                    height: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .height
                        .clone(),
                    length: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .dimensions
                        .clone()
                        .unwrap_or_default()
                        .length
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.schedule_environment_expiry_0_port)(outgoing_0).await?;
        let outgoing_1 = configuration::ListConfigSetReportsRequest {
            context: req.context.clone(),
            filter: Some(configuration::list_config_set_reports_request::Filter {
                created_after: reply_0
                    .search_index
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: (match reply_0.search_index.clone().unwrap_or_default().locator.clone() {
                    Some(search::search_index::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.schedule_environment_expiry_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = configuration::ScheduleEnvironmentExpiryResponse::default();
        let contribution: String = reply_0
            .search_index
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by TerminateEnvironment for NormalizeKeyRotation.
struct TerminateEnvironmentDispatch0 {
    context: Option<common::RequestContext>,
    key_rotation_participants_contact_preferred_channel: String,
}
impl From<configuration::TerminateEnvironmentRequest> for TerminateEnvironmentDispatch0 {
    fn from(req: configuration::TerminateEnvironmentRequest) -> Self {
        Self {
            context: req.context.clone(),
            key_rotation_participants_contact_preferred_channel: req
                .details
                .clone()
                .unwrap_or_default()
                .notify_contact
                .clone()
                .unwrap_or_default()
                .preferred_channel
                .clone(),
        }
    }
}
impl From<TerminateEnvironmentDispatch0> for keymanagement::NormalizeKeyRotationRequest {
    fn from(value: TerminateEnvironmentDispatch0) -> Self {
        keymanagement::NormalizeKeyRotationRequest {
            context: value.context,
            key_rotation: Some(keymanagement::KeyRotation {
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        preferred_channel: value.key_rotation_participants_contact_preferred_channel,
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by ExpireEnvironment for FindConfigChanges.
struct ExpireEnvironmentDispatch0 {
    context: Option<common::RequestContext>,
    filter_created_after: Option<prost_types::Timestamp>,
    filter_owner_id: String,
    filter_labels: std::collections::BTreeMap<String, String>,
}
impl From<configuration::ExpireEnvironmentRequest> for ExpireEnvironmentDispatch0 {
    fn from(req: configuration::ExpireEnvironmentRequest) -> Self {
        Self {
            context: req.context.clone(),
            filter_created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
            filter_owner_id: req
                .details
                .clone()
                .unwrap_or_default()
                .new_owner
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            filter_labels: req.annotations.clone(),
        }
    }
}
impl From<ExpireEnvironmentDispatch0> for configuration::FindConfigChangesRequest {
    fn from(value: ExpireEnvironmentDispatch0) -> Self {
        configuration::FindConfigChangesRequest {
            context: value.context,
            filter: Some(configuration::find_config_changes_request::Filter {
                created_after: value.filter_created_after,
                labels: value.filter_labels,
                owner_id: value.filter_owner_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl EnvironmentLifecycleService for EnvironmentLifecycleController {
    /// Activates an environment.
    async fn activate_environment(
        &self,
        request: tonic::Request<configuration::ActivateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::ActivateEnvironmentResponse>, tonic::Status> {
        self.run_activate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Suspends an environment.
    async fn suspend_environment(
        &self,
        request: tonic::Request<configuration::SuspendEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::SuspendEnvironmentResponse>, tonic::Status> {
        self.run_suspend_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Reinstates an environment.
    async fn reinstate_environment(
        &self,
        request: tonic::Request<configuration::ReinstateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::ReinstateEnvironmentResponse>, tonic::Status> {
        self.run_reinstate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Terminates an environment.
    async fn terminate_environment(
        &self,
        request: tonic::Request<configuration::TerminateEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::TerminateEnvironmentResponse>, tonic::Status> {
        self.run_terminate_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Renews an environment.
    async fn renew_environment(
        &self,
        request: tonic::Request<configuration::RenewEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::RenewEnvironmentResponse>, tonic::Status> {
        self.run_renew_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Expires an environment.
    async fn expire_environment(
        &self,
        request: tonic::Request<configuration::ExpireEnvironmentRequest>,
    ) -> Result<tonic::Response<configuration::ExpireEnvironmentResponse>, tonic::Status> {
        self.run_expire_environment(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns an environment.
    async fn get_environment_lifecycle(
        &self,
        request: tonic::Request<configuration::GetEnvironmentLifecycleRequest>,
    ) -> Result<tonic::Response<configuration::GetEnvironmentLifecycleResponse>, tonic::Status> {
        self.run_get_environment_lifecycle(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Schedules an environment.
    async fn schedule_environment_expiry(
        &self,
        request: tonic::Request<configuration::ScheduleEnvironmentExpiryRequest>,
    ) -> Result<tonic::Response<configuration::ScheduleEnvironmentExpiryResponse>, tonic::Status> {
        self.run_schedule_environment_expiry(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
