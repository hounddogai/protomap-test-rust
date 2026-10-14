use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_service_client::MetricSeriesServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::secrets::v1::access_grant_calculation_service_client::AccessGrantCalculationServiceClient;
use acme_proto::acme::secrets::v1::access_grant_validation_service_server::AccessGrantValidationService;

/// Checks access grants against the rules before anything acts on them.
#[derive(Clone)]
pub struct AccessGrantValidationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::SecretsRepository>,
    lint_access_grants_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            keymanagement::RecalculateKeyRotationsRequest,
            keymanagement::RecalculateKeyRotationsResponse,
        >,
    >,
    normalize_access_grant_0_port: acme_platform::ports::Callback<
        secrets::CompareAccessGrantOptionsRequest,
        secrets::CompareAccessGrantOptionsResponse,
    >,
    preview_access_grant_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<secrets::SimulateAccessGrantRequest, secrets::SimulateAccessGrantResponse>,
    >,
    preview_access_grant_1_port: acme_platform::ports::Callback<
        certificates::CheckCertificateAuthorityEligibilityRequest,
        certificates::CheckCertificateAuthorityEligibilityResponse,
    >,
}
impl AccessGrantValidationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::SecretsRepository>,
    ) -> Self {
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
        let preview_access_grant_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            certificates::CheckCertificateAuthorityEligibilityRequest,
            certificates::CheckCertificateAuthorityEligibilityResponse,
        > = {
            let client = CertificateAuthorityValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(
                move |request: certificates::CheckCertificateAuthorityEligibilityRequest| -> acme_platform::ports::RpcFuture<
                    certificates::CheckCertificateAuthorityEligibilityResponse,
                > {
                    let mut client = client.clone();
                    Box::pin(async move {
                        let context = request.context.clone().unwrap_or_default();
                        let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
                        Ok(client
                            .check_certificate_authority_eligibility(request)
                            .await?
                            .into_inner())
                    })
                },
            )
        };
        let preview_access_grant_1_port = callback;
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
        let normalize_access_grant_0_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::RecalculateKeyRotationsRequest,
            keymanagement::RecalculateKeyRotationsResponse,
        > = {
            let client = KeyRotationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::RecalculateKeyRotationsRequest| -> acme_platform::ports::RpcFuture<keymanagement::RecalculateKeyRotationsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_key_rotations(request).await?.into_inner())
})
})
        };
        let lint_access_grants_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            lint_access_grants_2_port,
            normalize_access_grant_0_port,
            preview_access_grant_0_port,
            preview_access_grant_1_port,
        }
    }
    /// Run ValidateAccessGrant after validating the tenant and durably recording the command.
    async fn run_validate_access_grant(
        &self,
        req: secrets::ValidateAccessGrantRequest,
    ) -> Result<secrets::ValidateAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::ExplainAccessGrantCalculationRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<secrets::ExplainAccessGrantCalculationRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = AccessGrantCalculationServiceClient::new(channel);
                let input: tonic::Request<secrets::ExplainAccessGrantCalculationRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_access_grant_calculation(input).await?;
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
        let outgoing_1 = certificates::ExplainCertificateAuthorityDecisionRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        manager: Some(identity::UserRef {
                            display_name: req
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ExplainCertificateAuthorityDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_certificate_authority_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = secrets::ValidateAccessGrantResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.certificate_authority.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.access_grant.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PreviewAccessGrant after validating the tenant and durably recording the command.
    async fn run_preview_access_grant(
        &self,
        req: secrets::PreviewAccessGrantRequest,
    ) -> Result<secrets::PreviewAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::SimulateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                owner: Some(identity::UserRef {
                    display_name: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
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
                    user_id: req
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
                action: Some(secrets::access_grant::Action::Route(secrets::AccessGrantRouteAction {
                    target: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Route(value)) => value,
                        _ => Default::default(),
                    })
                    .target
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 =
            self.preview_access_grant_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = certificates::CheckCertificateAuthorityEligibilityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        manager: Some(identity::UserRef {
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.preview_access_grant_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = audit::WatchAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::watch_audit_events_request::Filter {
                region: reply_1
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
            subscriber: Some(common::ContactInfo {
                email: reply_0
                    .access_grant
                    .clone()
                    .unwrap_or_default()
                    .owner
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
        };
        let channel = self.channel.clone();
        let _reply_2 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::WatchAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_audit_events(input).await?;
            let mut frames = response.into_inner();
            let mut last = audit::WatchAuditEventsResponse::default();
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
        let outgoing_3 = metrics::WatchMetricSeriesesRequest {
            context: req.context.clone(),
            filter: Some(metrics::watch_metric_serieses_request::Filter {
                region: reply_1
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
            subscriber: Some(common::ContactInfo {
                email: reply_1
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .manager
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_3 = async move {
            let mut client = MetricSeriesServiceClient::new(channel);
            let input: tonic::Request<metrics::WatchMetricSeriesesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_metric_serieses(input).await?;
            let mut frames = response.into_inner();
            let mut last = metrics::WatchMetricSeriesesResponse::default();
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
        let mut response = secrets::PreviewAccessGrantResponse::default();
        let contribution: i32 = reply_0.access_grant.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.access_grant.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run CheckAccessGrantEligibility after validating the tenant and durably recording the command.
    async fn run_check_access_grant_eligibility(
        &self,
        req: secrets::CheckAccessGrantEligibilityRequest,
    ) -> Result<secrets::CheckAccessGrantEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckAccessGrantEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckAccessGrantEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().name.clone(), "recorded audit field");
        let outgoing_0 = secrets::GetAccessGrantRatesRequest {
            access_grant_id: req.access_grant.clone().unwrap_or_default().access_grant_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<secrets::GetAccessGrantRatesRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
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
        let mut command_1 = CheckAccessGrantEligibilityDispatch1::from(req.clone());
        command_1.access_grant_owner_display_name = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        let outgoing_1: secrets::EstimateAccessGrantRequest = command_1.into();
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
        let outgoing_2 = certificates::ScoreCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                requester: Some(common::ContactInfo {
                    email: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .owner
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
            }),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::codec::through_any(&outgoing_2, "acme.certificates.v1.ScoreCsrRequest")?;
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CsrValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ScoreCsrRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = secrets::CalculateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    updated_by: req
                        .access_grant
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
                    email: reply_2
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: reply_2
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<secrets::CalculateAccessGrantRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = AccessGrantCalculationServiceClient::new(channel);
                let input: tonic::Request<secrets::CalculateAccessGrantRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.calculate_access_grant(input).await?;
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
        let mut response = secrets::CheckAccessGrantEligibilityResponse::default();
        let contribution: String = reply_0
            .access_grant
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.access_grant.clone().unwrap_or_default().parameters.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .parameters
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .csr
            .clone()
            .unwrap_or_default()
            .requester
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_3
            .access_grant
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Option<common::TimeWindow> =
            reply_3.access_grant.clone().unwrap_or_default().effective.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.access_grant.get_or_insert_with(Default::default).effective = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: i32 = req.access_grant.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeAccessGrant after validating the tenant and durably recording the command.
    async fn run_normalize_access_grant(
        &self,
        req: secrets::NormalizeAccessGrantRequest,
    ) -> Result<secrets::NormalizeAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::CompareAccessGrantOptionsRequest {
            access_grant: Some(secrets::AccessGrant {
                action: Some(secrets::access_grant::Action::Block(secrets::AccessGrantBlockAction {
                    notify_email: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Block(value)) => value,
                        _ => Default::default(),
                    })
                    .notify_email
                    .clone(),
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    user_id: req
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
        let reply_0 = (self.normalize_access_grant_0_port)(outgoing_0).await?;
        let mut response = secrets::NormalizeAccessGrantResponse::default();
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
                .access_grant
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: i32 = req.access_grant.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run ExplainAccessGrantDecision after validating the tenant and durably recording the command.
    async fn run_explain_access_grant_decision(
        &self,
        req: secrets::ExplainAccessGrantDecisionRequest,
    ) -> Result<secrets::ExplainAccessGrantDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainAccessGrantDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainAccessGrantDecision", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ExplainQuotaBucketCalculationRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .access_grant
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    etag: req
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
                limits: vec![ratelimiting::QuotaBucketLimit {
                    amount: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Discount(value)) => value,
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
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainQuotaBucketCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_quota_bucket_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = secrets::ExplainAccessGrantDecisionResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
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
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.access_grant.clone().unwrap_or_default().owner.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.access_grant.get_or_insert_with(Default::default).owner = contribution;
        }
        Ok(response)
    }
    /// Run ScoreAccessGrant after validating the tenant and durably recording the command.
    async fn run_score_access_grant(
        &self,
        req: secrets::ScoreAccessGrantRequest,
    ) -> Result<secrets::ScoreAccessGrantResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreAccessGrant", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreAccessGrant", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::QuoteAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                action: Some(secrets::access_grant::Action::Route(secrets::AccessGrantRouteAction {
                    target: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Route(value)) => value,
                        _ => Default::default(),
                    })
                    .target
                    .clone(),
                    ..Default::default()
                })),
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
        let mut response = secrets::ScoreAccessGrantResponse::default();
        let contribution: String = reply_0.access_grant.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.access_grant.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run LintAccessGrants after validating the tenant and durably recording the command.
    async fn run_lint_access_grants(
        &self,
        req: secrets::LintAccessGrantsRequest,
    ) -> Result<secrets::LintAccessGrantsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintAccessGrants", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintAccessGrants", "running workflow");
        tracing::debug!(audit_value = ?req.access_grant.clone().unwrap_or_default().access_grant_id.clone(), "recorded audit field");
        let outgoing_0 = secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                action: Some(secrets::access_grant::Action::Route(secrets::AccessGrantRouteAction {
                    target: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Route(value)) => value,
                        _ => Default::default(),
                    })
                    .target
                    .clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = AccessGrantCalculationServiceClient::new(channel);
            let input: tonic::Request<secrets::EstimateAccessGrantRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_access_grant(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = secrets::GetAccessGrantRatesRequest {
            access_grant_id: req.access_grant_id.clone(),
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
        let outgoing_2 = keymanagement::RecalculateKeyRotationsRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                participants: vec![keymanagement::KeyRotationParticipant {
                    contact: Some(common::ContactInfo {
                        email: reply_0
                            .access_grant
                            .clone()
                            .unwrap_or_default()
                            .owner
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
        let reply_2 = acme_platform::ports::execute(self.lint_access_grants_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = ratelimiting::CalculateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                limits: vec![ratelimiting::QuotaBucketLimit {
                    amount: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                        Some(secrets::access_grant::Action::Discount(value)) => value,
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
        let reply_3 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CalculateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = secrets::LintAccessGrantsResponse::default();
        let contribution: i32 = reply_0.access_grant.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_0.access_grant.clone().unwrap_or_default().scopes.clone();
        if contribution != <Vec<String>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .scopes
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .access_grant
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
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
                .access_grant
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.access_grant_id.clone();
        if contribution != <String>::default() {
            response
                .access_grant
                .get_or_insert_with(Default::default)
                .access_grant_id = contribution;
        }
        let contribution: i32 = req.access_grant.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.access_grant.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CheckAccessGrantEligibility for EstimateAccessGrant.
struct CheckAccessGrantEligibilityDispatch1 {
    context: Option<common::RequestContext>,
    access_grant_discount_amount: Option<common::Money>,
    access_grant_audit_created_at: Option<prost_types::Timestamp>,
    access_grant_owner_display_name: String,
}
impl From<secrets::CheckAccessGrantEligibilityRequest> for CheckAccessGrantEligibilityDispatch1 {
    fn from(req: secrets::CheckAccessGrantEligibilityRequest) -> Self {
        Self {
            context: req.context.clone(),
            access_grant_discount_amount: (match req.access_grant.clone().unwrap_or_default().action.clone() {
                Some(secrets::access_grant::Action::Discount(value)) => value,
                _ => Default::default(),
            })
            .amount
            .clone(),
            access_grant_audit_created_at: req
                .access_grant
                .clone()
                .unwrap_or_default()
                .audit
                .clone()
                .unwrap_or_default()
                .created_at
                .clone(),
            access_grant_owner_display_name: Default::default(),
        }
    }
}
impl From<CheckAccessGrantEligibilityDispatch1> for secrets::EstimateAccessGrantRequest {
    fn from(value: CheckAccessGrantEligibilityDispatch1) -> Self {
        secrets::EstimateAccessGrantRequest {
            access_grant: Some(secrets::AccessGrant {
                audit: Some(common::AuditInfo {
                    created_at: value.access_grant_audit_created_at,
                    ..Default::default()
                }),
                action: Some(secrets::access_grant::Action::Discount(
                    secrets::AccessGrantDiscountAction {
                        amount: value.access_grant_discount_amount,
                        ..Default::default()
                    },
                )),
                owner: Some(identity::UserRef {
                    display_name: value.access_grant_owner_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl AccessGrantValidationService for AccessGrantValidationEndpoint {
    /// Validates an access grant.
    async fn validate_access_grant(
        &self,
        request: tonic::Request<secrets::ValidateAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::ValidateAccessGrantResponse>, tonic::Status> {
        self.run_validate_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews an access grant.
    async fn preview_access_grant(
        &self,
        request: tonic::Request<secrets::PreviewAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::PreviewAccessGrantResponse>, tonic::Status> {
        self.run_preview_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks an access grant.
    async fn check_access_grant_eligibility(
        &self,
        request: tonic::Request<secrets::CheckAccessGrantEligibilityRequest>,
    ) -> Result<tonic::Response<secrets::CheckAccessGrantEligibilityResponse>, tonic::Status> {
        self.run_check_access_grant_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes an access grant.
    async fn normalize_access_grant(
        &self,
        request: tonic::Request<secrets::NormalizeAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::NormalizeAccessGrantResponse>, tonic::Status> {
        self.run_normalize_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains an access grant.
    async fn explain_access_grant_decision(
        &self,
        request: tonic::Request<secrets::ExplainAccessGrantDecisionRequest>,
    ) -> Result<tonic::Response<secrets::ExplainAccessGrantDecisionResponse>, tonic::Status> {
        self.run_explain_access_grant_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores an access grant.
    async fn score_access_grant(
        &self,
        request: tonic::Request<secrets::ScoreAccessGrantRequest>,
    ) -> Result<tonic::Response<secrets::ScoreAccessGrantResponse>, tonic::Status> {
        self.run_score_access_grant(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints an access grant.
    async fn lint_access_grants(
        &self,
        request: tonic::Request<secrets::LintAccessGrantsRequest>,
    ) -> Result<tonic::Response<secrets::LintAccessGrantsResponse>, tonic::Status> {
        self.run_lint_access_grants(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
