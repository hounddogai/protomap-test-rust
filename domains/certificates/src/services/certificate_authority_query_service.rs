use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_server::CertificateAuthorityQueryService;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_service_client::NotificationServiceClient;
use acme_proto::acme::performers::v1 as performers;
use acme_proto::acme::performers::v1::performer_service_client::PerformerServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_validation_service_client::RateLimitValidationServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_service_client::VenueServiceClient;

/// Answers read-only questions about certificate authorities, for screens and reports.
#[derive(Clone)]
pub struct CertificateAuthorityQueryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    browse_certificate_authorities_1_port:
        std::sync::Arc<acme_platform::ports::Adapter<certificates::ScoreCsrRequest, certificates::ScoreCsrResponse>>,
    resolve_certificate_authorities_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            certificates::NormalizeCertificateAuthorityRequest,
            certificates::NormalizeCertificateAuthorityResponse,
        >,
    >,
    resolve_certificate_authorities_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<certificates::SimulateCsrRequest, certificates::SimulateCsrResponse>,
    >,
    suggest_certificate_authorities_0_port:
        acme_platform::ports::Callback<certificates::LintCsrsRequest, certificates::LintCsrsResponse>,
    summarize_certificate_authorities_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<certificates::CalculateCsrRequest, certificates::CalculateCsrResponse>,
    >,
}
impl CertificateAuthorityQueryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<certificates::ScoreCsrRequest, certificates::ScoreCsrResponse> = {
            let client = CsrValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::ScoreCsrRequest| -> acme_platform::ports::RpcFuture<certificates::ScoreCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_csr(request).await?.into_inner())
})
})
        };
        let browse_certificate_authorities_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::NormalizeCertificateAuthorityRequest,
            certificates::NormalizeCertificateAuthorityResponse,
        > = {
            let client = CertificateAuthorityValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::NormalizeCertificateAuthorityRequest| -> acme_platform::ports::RpcFuture<certificates::NormalizeCertificateAuthorityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_certificate_authority(request).await?.into_inner())
})
})
        };
        let resolve_certificate_authorities_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::SimulateCsrRequest,
            certificates::SimulateCsrResponse,
        > = {
            let client = CsrCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::SimulateCsrRequest| -> acme_platform::ports::RpcFuture<certificates::SimulateCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_csr(request).await?.into_inner())
})
})
        };
        let resolve_certificate_authorities_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::CalculateCsrRequest,
            certificates::CalculateCsrResponse,
        > = {
            let client = CsrCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CalculateCsrRequest| -> acme_platform::ports::RpcFuture<certificates::CalculateCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_csr(request).await?.into_inner())
})
})
        };
        let summarize_certificate_authorities_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<certificates::LintCsrsRequest, certificates::LintCsrsResponse> = {
            let client = CsrValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::LintCsrsRequest| -> acme_platform::ports::RpcFuture<certificates::LintCsrsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lint_csrs(request).await?.into_inner())
})
})
        };
        let suggest_certificate_authorities_0_port = callback;
        Self {
            channel,
            repository,
            browse_certificate_authorities_1_port,
            resolve_certificate_authorities_0_port,
            resolve_certificate_authorities_1_port,
            suggest_certificate_authorities_0_port,
            summarize_certificate_authorities_0_port,
        }
    }
    /// Run LookupCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_lookup_certificate_authority(
        &self,
        req: certificates::LookupCertificateAuthorityRequest,
    ) -> Result<certificates::LookupCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("LookupCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupCertificateAuthority", "running workflow");
        let outgoing_0 = ratelimiting::CheckRateLimitEligibilityRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::check_rate_limit_eligibility_request::Details {
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
            rate_limit: Some(ratelimiting::RateLimit {
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
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.ratelimiting.v1.CheckRateLimitEligibilityRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CheckRateLimitEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_rate_limit_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = notifications::BatchCreateNotificationsRequest {
            context: req.context.clone(),
            chunk: Some(notifications::batch_create_notifications_request::Chunk::Record(
                notifications::Notification {
                    assignee: Some(identity::UserRef {
                        email: reply_0
                            .rate_limit
                            .clone()
                            .unwrap_or_default()
                            .owner
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        user_id: reply_0
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
                    audit: Some(common::AuditInfo {
                        etag: reply_0
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
                    origin: Some(notifications::notification::Origin::Customer(
                        notifications::NotificationTemplateRef {
                            display_name: reply_0
                                .rate_limit
                                .clone()
                                .unwrap_or_default()
                                .owner
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = NotificationServiceClient::new(channel);
            let input: tonic::Request<notifications::BatchCreateNotificationsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_notifications(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    owner: Some(identity::UserRef {
                        display_name: reply_0
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_feature_flag_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFeatureFlagChangesResponse::default();
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
        let mut response = certificates::LookupCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_0.rate_limit.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_browse_certificate_authorities(
        &self,
        req: certificates::BrowseCertificateAuthoritiesRequest,
    ) -> Result<certificates::BrowseCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("BrowseCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = venues::CreateVenueRequest {
            context: req.context.clone(),
            venue: Some(venues::Venue {
                locator: Some(venues::venue::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VenueServiceClient::new(channel);
            let input: tonic::Request<venues::CreateVenueRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_venue(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::ScoreCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
                    display_name: reply_0
                        .venue
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            details: Some(certificates::score_csr_request::Details {
                destination: Some(common::PostalAddress {
                    locality: (match reply_0.venue.clone().unwrap_or_default().locator.clone() {
                        Some(venues::venue::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .locality
                    .clone(),
                    region: (match reply_0.venue.clone().unwrap_or_default().locator.clone() {
                        Some(venues::venue::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .region
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.browse_certificate_authorities_1_port.as_ref(), outgoing_1).await?;
        let mut response = certificates::BrowseCertificateAuthoritiesResponse::default();
        let contribution: String = (match reply_0.venue.clone().unwrap_or_default().locator.clone() {
            Some(venues::venue::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .recipient
        .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            if response.certificate_authorities[0].children.is_empty() {
                response.certificate_authorities[0].children.push(Default::default());
            }
            response.certificate_authorities[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
        }
        let contribution: String = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            response.certificate_authorities[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run FindCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_find_certificate_authorities(
        &self,
        req: certificates::FindCertificateAuthoritiesRequest,
    ) -> Result<certificates::FindCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("FindCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let command_0 = FindCertificateAuthoritiesDispatch0::from(req.clone());
        let outgoing_0: featureflags::UnsubscribeFlagRolloutUpdatesRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::UnsubscribeFlagRolloutUpdatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.unsubscribe_flag_rollout_updates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::FindCertificateAuthoritiesResponse::default();
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .recipient
            .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            if response.certificate_authorities[0].children.is_empty() {
                response.certificate_authorities[0].children.push(Default::default());
            }
            if response.certificate_authorities[0].children[0].children.is_empty() {
                response.certificate_authorities[0].children[0]
                    .children
                    .push(Default::default());
            }
            response.certificate_authorities[0].children[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
        }
        let contribution: String = reply_0
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            if response.certificate_authorities[0].children.is_empty() {
                response.certificate_authorities[0].children.push(Default::default());
            }
            response.certificate_authorities[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        Ok(response)
    }
    /// Run CountCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_count_certificate_authorities(
        &self,
        req: certificates::CountCertificateAuthoritiesRequest,
    ) -> Result<certificates::CountCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CountCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = featureflags::PreviewFlagRolloutRequest {
            context: req.context.clone(),
            details: Some(featureflags::preview_flag_rollout_request::Details {
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
            flag_rollout: Some(featureflags::FlagRollout {
                location: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = certificates::ResolveCertificatesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = FlagRolloutValidationServiceClient::new(channel_0);
                let input: tonic::Request<featureflags::PreviewFlagRolloutRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.preview_flag_rollout(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CertificateQueryServiceClient::new(channel_1);
                let input: tonic::Request<certificates::ResolveCertificatesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_certificates(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = certificates::CountCertificateAuthoritiesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ResolveCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_resolve_certificate_authorities(
        &self,
        req: certificates::ResolveCertificateAuthoritiesRequest,
    ) -> Result<certificates::ResolveCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveCertificateAuthorities", "running workflow");
        let outgoing_0 = certificates::NormalizeCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        manager: Some(identity::UserRef {
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
                    }],
                    manager: Some(identity::UserRef {
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::normalize_certificate_authority_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.resolve_certificate_authorities_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = certificates::SimulateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
                    email: reply_0
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    updated_at: reply_0
                        .certificate_authority
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
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.resolve_certificate_authorities_1_port.as_ref(), outgoing_1).await?;
        let mut response = certificates::ResolveCertificateAuthoritiesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = (match reply_1.csr.clone().unwrap_or_default().origin.clone() {
            Some(certificates::csr::Origin::Customer(value)) => value,
            _ => Default::default(),
        })
        .email
        .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            response.certificate_authorities[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run ExportCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_export_certificate_authorities(
        &self,
        req: certificates::ExportCertificateAuthoritiesRequest,
    ) -> Result<certificates::ExportCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExportCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().owner_id.clone(), "recorded audit field");
        let outgoing_0 = performers::ListPerformersRequest {
            context: req.context.clone(),
            filter: Some(performers::list_performers_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = certificates::EstimateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = PerformerServiceClient::new(channel_0);
                let input: tonic::Request<performers::ListPerformersRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_performers(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CsrCalculationServiceClient::new(channel_1);
                let input: tonic::Request<certificates::EstimateCsrRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.estimate_csr(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = certificates::ValidateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    audit: Some(common::AuditInfo {
                        created_by: reply_1
                            .csr
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        ..Default::default()
                    }),
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            updated_at: reply_1
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            email: reply_1
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .requester
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            user_id: reply_1
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_2 =
            acme_platform::ports::RequestBuilder::<certificates::ValidateCertificateAuthorityRequest>::default()
                .map(|draft| *draft = outgoing_2)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ValidateCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::ExportCertificateAuthoritiesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .csr
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Vec<String> = reply_2
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .lines
            .clone();
        if contribution != <Vec<String>>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children[0]
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .children[0]
                .location
                .get_or_insert_with(Default::default)
                .lines
                .extend(contribution);
        }
        let contribution: String = reply_2
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_summarize_certificate_authorities(
        &self,
        req: certificates::SummarizeCertificateAuthoritiesRequest,
    ) -> Result<certificates::SummarizeCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SummarizeCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CalculateCsrRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.summarize_certificate_authorities_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: reply_0
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            email: reply_0
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .requester
                                .clone()
                                .unwrap_or_default()
                                .email
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        display_name: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                            Some(certificates::csr::Origin::Customer(value)) => value,
                            _ => Default::default(),
                        })
                        .display_name
                        .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::RecalculateConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ValidateRateLimitRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::validate_rate_limit_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_1
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
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
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: reply_0
                        .csr
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
                    email: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .email
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<ratelimiting::ValidateRateLimitRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = RateLimitValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ValidateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::SummarizeCertificateAuthoritiesResponse::default();
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_suggest_certificate_authorities(
        &self,
        req: certificates::SuggestCertificateAuthoritiesRequest,
    ) -> Result<certificates::SuggestCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SuggestCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::LintCsrsRequest {
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
            details: Some(certificates::lint_csrs_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.suggest_certificate_authorities_0_port)(outgoing_0).await?;
        let mut response = certificates::SuggestCertificateAuthoritiesResponse::default();
        let contribution: String = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .requester
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response.certificate_authorities.is_empty() {
                response.certificate_authorities.push(Default::default());
            }
            if response.certificate_authorities[0].children.is_empty() {
                response.certificate_authorities[0].children.push(Default::default());
            }
            response.certificate_authorities[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run PeekCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_peek_certificate_authority(
        &self,
        req: certificates::PeekCertificateAuthorityRequest,
    ) -> Result<certificates::PeekCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PeekCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekCertificateAuthority", "running workflow");
        let outgoing_0 = certificates::EstimateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        children: vec![certificates::CertificateAuthority {
                            certificate_authority_id: req.certificate_authority_id.clone(),
                            ..Default::default()
                        }],
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::EstimateCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::ListFlagEvaluationVersionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_evaluation_versions_request::Filter {
                created_after: reply_0
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: reply_0
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .location
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagEvaluationHistoryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFlagEvaluationVersionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_flag_evaluation_versions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::PeekCertificateAuthorityResponse::default();
        let contribution: Vec<String> = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .lines
            .clone();
        if contribution != <Vec<String>>::default() {
            if response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .certificate_authority
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .location
                .get_or_insert_with(Default::default)
                .lines
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
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by FindCertificateAuthorities for UnsubscribeFlagRolloutUpdates.
struct FindCertificateAuthoritiesDispatch0 {
    context: Option<common::RequestContext>,
    details_recipient_email: String,
    details_variables: std::collections::BTreeMap<String, String>,
    annotations: std::collections::BTreeMap<String, String>,
}
impl From<certificates::FindCertificateAuthoritiesRequest> for FindCertificateAuthoritiesDispatch0 {
    fn from(req: certificates::FindCertificateAuthoritiesRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_recipient_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            details_variables: req.filter.clone().unwrap_or_default().labels.clone(),
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
        }
    }
}
impl From<FindCertificateAuthoritiesDispatch0> for featureflags::UnsubscribeFlagRolloutUpdatesRequest {
    fn from(value: FindCertificateAuthoritiesDispatch0) -> Self {
        featureflags::UnsubscribeFlagRolloutUpdatesRequest {
            annotations: value.annotations,
            context: value.context,
            details: Some(featureflags::unsubscribe_flag_rollout_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    email: value.details_recipient_email,
                    ..Default::default()
                }),
                variables: value.details_variables,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CertificateAuthorityQueryService for CertificateAuthorityQueryEndpoint {
    /// Returns a certificate authority.
    async fn lookup_certificate_authority(
        &self,
        request: tonic::Request<certificates::LookupCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::LookupCertificateAuthorityResponse>, tonic::Status> {
        self.run_lookup_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificate authorities a page at a time.
    async fn browse_certificate_authorities(
        &self,
        request: tonic::Request<certificates::BrowseCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::BrowseCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_browse_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificate authorities a page at a time.
    async fn find_certificate_authorities(
        &self,
        request: tonic::Request<certificates::FindCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::FindCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_find_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts certificate authorities by status.
    async fn count_certificate_authorities(
        &self,
        request: tonic::Request<certificates::CountCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::CountCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_count_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several certificate authorities at once.
    async fn resolve_certificate_authorities(
        &self,
        request: tonic::Request<certificates::ResolveCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::ResolveCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_resolve_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportCertificateAuthoritiesStream = tokio_stream::wrappers::ReceiverStream<
        Result<certificates::ExportCertificateAuthoritiesResponse, tonic::Status>,
    >;
    /// Streams every matching certificate authority.
    async fn export_certificate_authorities(
        &self,
        request: tonic::Request<certificates::ExportCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<Self::ExportCertificateAuthoritiesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream =
            acme_platform::transport::one_reply(
                async move { handler.run_export_certificate_authorities(request).await },
            );
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a certificate authority.
    async fn summarize_certificate_authorities(
        &self,
        request: tonic::Request<certificates::SummarizeCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::SummarizeCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_summarize_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificate authorities a page at a time.
    async fn suggest_certificate_authorities(
        &self,
        request: tonic::Request<certificates::SuggestCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::SuggestCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_suggest_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a certificate authority.
    async fn peek_certificate_authority(
        &self,
        request: tonic::Request<certificates::PeekCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::PeekCertificateAuthorityResponse>, tonic::Status> {
        self.run_peek_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
