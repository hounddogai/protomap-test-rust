use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_server::CertificateQueryService;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_service_client::FeatureFlagServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_client::QuotaBucketQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_calculation_service_client::ThrottleCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;

/// Answers read-only questions about certificates, for screens and reports.
#[derive(Clone)]
pub struct CertificateQueryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    browse_certificates_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<certificates::PreviewCsrRequest, certificates::PreviewCsrResponse>,
    >,
    count_certificates_0_port: acme_platform::ports::Callback<
        certificates::RecalculateCertificatesRequest,
        certificates::RecalculateCertificatesResponse,
    >,
    suggest_certificates_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::RecalculateConfigSetsRequest,
                configuration::RecalculateConfigSetsResponse,
            >,
    >,
    summarize_certificates_0_port:
        acme_platform::ports::Callback<ratelimiting::CalculateThrottleRequest, ratelimiting::CalculateThrottleResponse>,
    summarize_certificates_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<ratelimiting::FindQuotaBucketsRequest, ratelimiting::FindQuotaBucketsResponse>,
    >,
}
impl CertificateQueryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::PreviewCsrRequest,
            certificates::PreviewCsrResponse,
        > = {
            let client = CsrValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::PreviewCsrRequest| -> acme_platform::ports::RpcFuture<certificates::PreviewCsrResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_csr(request).await?.into_inner())
})
})
        };
        let browse_certificates_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            certificates::RecalculateCertificatesRequest,
            certificates::RecalculateCertificatesResponse,
        > = {
            let client = CertificateCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::RecalculateCertificatesRequest| -> acme_platform::ports::RpcFuture<certificates::RecalculateCertificatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_certificates(request).await?.into_inner())
})
})
        };
        let count_certificates_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::CalculateThrottleRequest,
            ratelimiting::CalculateThrottleResponse,
        > = {
            let client = ThrottleCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CalculateThrottleRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CalculateThrottleResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_throttle(request).await?.into_inner())
})
})
        };
        let summarize_certificates_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::FindQuotaBucketsRequest,
            ratelimiting::FindQuotaBucketsResponse,
        > = {
            let client = QuotaBucketQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::FindQuotaBucketsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::FindQuotaBucketsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.find_quota_buckets(request).await?.into_inner())
})
})
        };
        let summarize_certificates_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::RecalculateConfigSetsRequest,
            configuration::RecalculateConfigSetsResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::RecalculateConfigSetsRequest| -> acme_platform::ports::RpcFuture<configuration::RecalculateConfigSetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_config_sets(request).await?.into_inner())
})
})
        };
        let suggest_certificates_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            browse_certificates_0_port,
            count_certificates_0_port,
            suggest_certificates_1_port,
            summarize_certificates_0_port,
            summarize_certificates_1_port,
        }
    }
    /// Run LookupCertificate after validating the tenant and durably recording the command.
    async fn run_lookup_certificate(
        &self,
        req: certificates::LookupCertificateRequest,
    ) -> Result<certificates::LookupCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupCertificate", "running workflow");
        let outgoing_0 = featureflags::BatchCreateFeatureFlagsRequest {
            context: req.context.clone(),
            chunk: Some(featureflags::batch_create_feature_flags_request::Chunk::Record(
                featureflags::FeatureFlag {
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagServiceClient::new(channel);
            let input: tonic::Request<featureflags::BatchCreateFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    attributes: req.context.clone().unwrap_or_default().baggage.clone(),
                    correlation_id: req.certificate_id.clone(),
                    measurements: vec![featureflags::FlagEvaluationMeasurement {
                        observed_at: req.context.clone().unwrap_or_default().deadline.clone(),
                        ..Default::default()
                    }],
                    source: req
                        .context
                        .clone()
                        .unwrap_or_default()
                        .actor
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = certificates::LookupCertificateResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseCertificates after validating the tenant and durably recording the command.
    async fn run_browse_certificates(
        &self,
        req: certificates::BrowseCertificatesRequest,
    ) -> Result<certificates::BrowseCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::PreviewCsrRequest {
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
                history: vec![certificates::CsrStatusChange {
                    changed_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            details: Some(certificates::preview_csr_request::Details {
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
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.browse_certificates_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = certificates::BrowseCertificatesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .sla
            .clone()
            .unwrap_or_default()
            .due_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            response.certificates[0].due_at = contribution;
        }
        Ok(response)
    }
    /// Run CountCertificates after validating the tenant and durably recording the command.
    async fn run_count_certificates(
        &self,
        req: certificates::CountCertificatesRequest,
    ) -> Result<certificates::CountCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::RecalculateCertificatesRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.count_certificates_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = certificates::CountCertificatesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run ResolveCertificates after validating the tenant and durably recording the command.
    async fn run_resolve_certificates(
        &self,
        req: certificates::ResolveCertificatesRequest,
    ) -> Result<certificates::ResolveCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveCertificates", "running workflow");
        let response = certificates::ResolveCertificatesResponse::default();
        Ok(response)
    }
    /// Run ExportCertificates after validating the tenant and durably recording the command.
    async fn run_export_certificates(
        &self,
        req: certificates::ExportCertificatesRequest,
    ) -> Result<certificates::ExportCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::BatchCreateConfigSetsRequest {
            context: req.context.clone(),
            chunk: Some(configuration::batch_create_config_sets_request::Chunk::Record(
                configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            region: req.filter.clone().unwrap_or_default().region.clone(),
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::BatchCreateConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_1 = ExportCertificatesDispatch1::from(req.clone());
        let outgoing_1: configuration::QuoteConfigSetRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::QuoteConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = ratelimiting::ValidateThrottleRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                attributes: (reply_1.config_set.clone().unwrap_or_default().attributes.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: reply_1
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .manager
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
        let reply_2 = async move {
            let mut client = ThrottleValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ValidateThrottleRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_throttle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::UpdateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            user_id: reply_1
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .manager
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::UpdateConfigSetRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetServiceClient::new(channel);
                let input: tonic::Request<configuration::UpdateConfigSetRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.update_config_set(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_3)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_3 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = certificates::ExportCertificatesResponse::default();
        let contribution: Option<common::Money> = reply_1.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.certificate.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3
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
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeCertificates after validating the tenant and durably recording the command.
    async fn run_summarize_certificates(
        &self,
        req: certificates::SummarizeCertificatesRequest,
    ) -> Result<certificates::SummarizeCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CalculateThrottleRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            reason: req.reason.clone(),
            throttle: Some(ratelimiting::Throttle {
                measurements: vec![ratelimiting::ThrottleMeasurement {
                    observed_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.summarize_certificates_0_port)(outgoing_0).await?;
        let outgoing_1 = ratelimiting::FindQuotaBucketsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::find_quota_buckets_request::Filter {
                created_after: reply_0.throttle.clone().unwrap_or_default().occurred_at.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0
                    .throttle
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .rate_limit_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            self.summarize_certificates_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = certificates::SummarizeCertificatesResponse::default();
        let contribution: String = reply_0.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestCertificates after validating the tenant and durably recording the command.
    async fn run_suggest_certificates(
        &self,
        req: certificates::SuggestCertificatesRequest,
    ) -> Result<certificates::SuggestCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = audit::CountAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::count_audit_events_request::Filter {
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AuditEventQueryServiceClient::new(channel);
            let input: tonic::Request<audit::CountAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::RecalculateConfigSetsRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            region: req.filter.clone().unwrap_or_default().region.clone(),
                            ..Default::default()
                        }),
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.suggest_certificates_1_port.as_ref(), outgoing_1).await?;
        let mut response = certificates::SuggestCertificatesResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
    /// Run PeekCertificate after validating the tenant and durably recording the command.
    async fn run_peek_certificate(
        &self,
        req: certificates::PeekCertificateRequest,
    ) -> Result<certificates::PeekCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekCertificate", "running workflow");
        let outgoing_0 = audit::ResolveAuditEventsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AuditEventQueryServiceClient::new(channel);
            let input: tonic::Request<audit::ResolveAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_audit_events(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::PeekCertificateResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ExportCertificates for QuoteConfigSet.
struct ExportCertificatesDispatch1 {
    context: Option<common::RequestContext>,
    config_set_children_location_region: String,
    config_set_children_children_manager_email: String,
}
impl From<certificates::ExportCertificatesRequest> for ExportCertificatesDispatch1 {
    fn from(req: certificates::ExportCertificatesRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_location_region: req.filter.clone().unwrap_or_default().region.clone(),
            config_set_children_children_manager_email: req
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
impl From<ExportCertificatesDispatch1> for configuration::QuoteConfigSetRequest {
    fn from(value: ExportCertificatesDispatch1) -> Self {
        configuration::QuoteConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            email: value.config_set_children_children_manager_email,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    location: Some(common::PostalAddress {
                        region: value.config_set_children_location_region,
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CertificateQueryService for CertificateQueryEndpoint {
    /// Returns a certificate.
    async fn lookup_certificate(
        &self,
        request: tonic::Request<certificates::LookupCertificateRequest>,
    ) -> Result<tonic::Response<certificates::LookupCertificateResponse>, tonic::Status> {
        self.run_lookup_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificates a page at a time.
    async fn browse_certificates(
        &self,
        request: tonic::Request<certificates::BrowseCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::BrowseCertificatesResponse>, tonic::Status> {
        self.run_browse_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts certificates by status.
    async fn count_certificates(
        &self,
        request: tonic::Request<certificates::CountCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::CountCertificatesResponse>, tonic::Status> {
        self.run_count_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several certificates at once.
    async fn resolve_certificates(
        &self,
        request: tonic::Request<certificates::ResolveCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::ResolveCertificatesResponse>, tonic::Status> {
        self.run_resolve_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportCertificatesStream =
        tokio_stream::wrappers::ReceiverStream<Result<certificates::ExportCertificatesResponse, tonic::Status>>;
    /// Streams every matching certificate.
    async fn export_certificates(
        &self,
        request: tonic::Request<certificates::ExportCertificatesRequest>,
    ) -> Result<tonic::Response<Self::ExportCertificatesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_export_certificates(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a certificate.
    async fn summarize_certificates(
        &self,
        request: tonic::Request<certificates::SummarizeCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::SummarizeCertificatesResponse>, tonic::Status> {
        self.run_summarize_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificates a page at a time.
    async fn suggest_certificates(
        &self,
        request: tonic::Request<certificates::SuggestCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::SuggestCertificatesResponse>, tonic::Status> {
        self.run_suggest_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a certificate.
    async fn peek_certificate(
        &self,
        request: tonic::Request<certificates::PeekCertificateRequest>,
    ) -> Result<tonic::Response<certificates::PeekCertificateResponse>, tonic::Status> {
        self.run_peek_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
