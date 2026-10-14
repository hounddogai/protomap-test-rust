use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_service_client::AdmissionServiceClient;
use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_service_server::CertificateService;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_query_service_client::LedgerAccountQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::rate_limit_query_service_client::RateLimitQueryServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_validation_service_client::SearchIndexValidationServiceClient;
use acme_proto::acme::tours::v1 as tours;
use acme_proto::acme::tours::v1::tour_query_service_client::TourQueryServiceClient;

/// Manages certificates: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct CertificateHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    delete_certificate_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::BrowseRateLimitsRequest, ratelimiting::BrowseRateLimitsResponse>,
    >,
    get_certificate_0_port:
        acme_platform::ports::Callback<configuration::CloneConfigSetRequest, configuration::CloneConfigSetResponse>,
    list_certificates_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<certificates::LintCsrsRequest, certificates::LintCsrsResponse>,
    >,
    list_certificates_1_port: acme_platform::ports::Callback<
        certificates::PreviewCertificateRequest,
        certificates::PreviewCertificateResponse,
    >,
}
impl CertificateHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::CloneConfigSetRequest,
            configuration::CloneConfigSetResponse,
        > = {
            let client = ConfigSetServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CloneConfigSetRequest| -> acme_platform::ports::RpcFuture<configuration::CloneConfigSetResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_config_set(request).await?.into_inner())
})
})
        };
        let get_certificate_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::BrowseRateLimitsRequest,
            ratelimiting::BrowseRateLimitsResponse,
        > = {
            let client = RateLimitQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::BrowseRateLimitsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::BrowseRateLimitsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_rate_limits(request).await?.into_inner())
})
})
        };
        let delete_certificate_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let list_certificates_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            certificates::PreviewCertificateRequest,
            certificates::PreviewCertificateResponse,
        > = {
            let client = CertificateValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::PreviewCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::PreviewCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_certificate(request).await?.into_inner())
})
})
        };
        let list_certificates_1_port = callback;
        Self {
            channel,
            repository,
            delete_certificate_0_port,
            get_certificate_0_port,
            list_certificates_0_port,
            list_certificates_1_port,
        }
    }
    /// Run CreateCertificate after validating the tenant and durably recording the command.
    async fn run_create_certificate(
        &self,
        req: certificates::CreateCertificateRequest,
    ) -> Result<certificates::CreateCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().billing_address.clone().unwrap_or_default().recipient.clone(), "recorded audit field");
        let outgoing_0 = certificates::GetCertificateRatesRequest {
            certificate_id: req.certificate.clone().unwrap_or_default().certificate_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<certificates::GetCertificateRatesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CertificateCalculationServiceClient::new(channel);
                let input: tonic::Request<certificates::GetCertificateRatesRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.get_certificate_rates(input).await?;
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
        let outgoing_1 = certificates::QuoteCertificateRequest {
            context: req.context.clone(),
            details: Some(certificates::quote_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::QuoteCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = admissions::BatchCreateAdmissionsRequest {
            context: req.context.clone(),
            chunk: Some(admissions::batch_create_admissions_request::Chunk::Record(
                admissions::Admission {
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    subject: Some(admissions::GateScanRef {
                        display_name: reply_0
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .counterparty
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = AdmissionServiceClient::new(channel);
            let input: tonic::Request<admissions::BatchCreateAdmissionsRequest> =
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
            let response = client.batch_create_admissions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    audit: Some(common::AuditInfo {
                        updated_by: reply_1
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_by
                            .clone(),
                        ..Default::default()
                    }),
                    location: Some(common::PostalAddress {
                        locality: req
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .billing_address
                            .clone()
                            .unwrap_or_default()
                            .locality
                            .clone(),
                        postal_code: req
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .billing_address
                            .clone()
                            .unwrap_or_default()
                            .postal_code
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
        let reply_3 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.stream_flag_rollout_changes(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagRolloutChangesResponse::default();
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
        let mut response = certificates::CreateCertificateResponse::default();
        let contribution: Option<common::PostalAddress> =
            reply_0.certificate.clone().unwrap_or_default().billing_address.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate.clone().unwrap_or_default().number.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).number = contribution;
        }
        let contribution: Option<certificates::CsrRef> =
            req.certificate.clone().unwrap_or_default().counterparty.clone();
        if contribution != <Option<certificates::CsrRef>>::default() {
            response.certificate.get_or_insert_with(Default::default).counterparty = contribution;
        }
        Ok(response)
    }
    /// Run GetCertificate after validating the tenant and durably recording the command.
    async fn run_get_certificate(
        &self,
        req: certificates::GetCertificateRequest,
    ) -> Result<certificates::GetCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCertificate", "running workflow");
        let outgoing_0 = configuration::CloneConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
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
            ..Default::default()
        };
        let reply_0 = (self.get_certificate_0_port)(outgoing_0).await?;
        let mut response = certificates::GetCertificateResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateCertificate after validating the tenant and durably recording the command.
    async fn run_update_certificate(
        &self,
        req: certificates::UpdateCertificateRequest,
    ) -> Result<certificates::UpdateCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().number.clone(), "recorded audit field");
        let command_0 = UpdateCertificateDispatch0::from(req.clone());
        let outgoing_0: search::CheckSearchIndexEligibilityRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = SearchIndexValidationServiceClient::new(channel);
            let input: tonic::Request<search::CheckSearchIndexEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_search_index_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = authorization::ListAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_access_policies_request::Filter {
                created_after: reply_0.outcome.clone().unwrap_or_default().effective_at.clone(),
                region: req
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .region
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.authorization.v1.ListAccessPoliciesRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AccessPolicyServiceClient::new(channel);
            let input: tonic::Request<authorization::ListAccessPoliciesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_access_policies(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::UpdateCertificateResponse::default();
        let contribution: Vec<String> = (match reply_0.search_index.clone().unwrap_or_default().locator.clone() {
            Some(search::search_index::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .lines
        .clone();
        if contribution != <Vec<String>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.certificate.clone().unwrap_or_default().certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<common::Money> = req.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.certificate.get_or_insert_with(Default::default).total = contribution;
        }
        Ok(response)
    }
    /// Run DeleteCertificate after validating the tenant and durably recording the command.
    async fn run_delete_certificate(
        &self,
        req: certificates::DeleteCertificateRequest,
    ) -> Result<certificates::DeleteCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.reason.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::BrowseRateLimitsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::browse_rate_limits_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                owner_id: req.certificate_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.delete_certificate_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = audit::WatchAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::watch_audit_events_request::Filter {
                owner_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                ..Default::default()
            }),
            subscriber: Some(common::ContactInfo {
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
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::WatchAuditEventsRequest> = tonic::IntoRequest::into_request(outgoing_1);
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
        let mut response = certificates::DeleteCertificateResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ListCertificates after validating the tenant and durably recording the command.
    async fn run_list_certificates(
        &self,
        req: certificates::ListCertificatesRequest,
    ) -> Result<certificates::ListCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::LintCsrsRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.list_certificates_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = certificates::PreviewCertificateRequest {
            context: req.context.clone(),
            details: Some(certificates::preview_certificate_request::Details {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
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
        let port = acme_platform::ports::opaque(self.list_certificates_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = certificates::ListCertificatesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            response.certificates[0]
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        Ok(response)
    }
    /// Run BatchGetCertificates after validating the tenant and durably recording the command.
    async fn run_batch_get_certificates(
        &self,
        req: certificates::BatchGetCertificatesRequest,
    ) -> Result<certificates::BatchGetCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetCertificates", "running workflow");
        let response = certificates::BatchGetCertificatesResponse::default();
        Ok(response)
    }
    /// Run SearchCertificates after validating the tenant and durably recording the command.
    async fn run_search_certificates(
        &self,
        req: certificates::SearchCertificatesRequest,
    ) -> Result<certificates::SearchCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SearchCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SearchCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::RecalculateCertificatesRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<certificates::RecalculateCertificatesRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::RecalculateCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::SearchCertificatesResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.certificate.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            response.certificates[0].due_at = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            reply_0.certificate.clone().unwrap_or_default().billing_address.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            if response.certificates.is_empty() {
                response.certificates.push(Default::default());
            }
            response.certificates[0].billing_address = contribution;
        }
        Ok(response)
    }
    /// Run WatchCertificates after validating the tenant and durably recording the command.
    async fn run_watch_certificates(
        &self,
        req: certificates::WatchCertificatesRequest,
    ) -> Result<certificates::WatchCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = configuration::LintEnvironmentsRequest {
            context: req.context.clone(),
            details: Some(configuration::lint_environments_request::Details {
                contact: Some(common::ContactInfo {
                    email: req.subscriber.clone().unwrap_or_default().email.clone(),
                    name: req.subscriber.clone().unwrap_or_default().name.clone(),
                    preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            environment: Some(configuration::Environment {
                locator: Some(configuration::environment::Locator::Address(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::LintEnvironmentsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = EnvironmentValidationServiceClient::new(channel);
                let input: tonic::Request<configuration::LintEnvironmentsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.lint_environments(input).await?;
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
        let outgoing_1 = certificates::RecalculateCertificateAuthoritiesRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            updated_at: reply_0
                                .environment
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
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::RecalculateCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_certificate_authorities(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = certificates::ExplainCertificateAuthorityDecisionRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            created_by: reply_1
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            updated_by: reply_1
                                .certificate_authority
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
            let mut client = CertificateAuthorityValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::ExplainCertificateAuthorityDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_certificate_authority_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::WatchCertificatesResponse::default();
        let contribution: Vec<String> = (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .lines
        .clone();
        if contribution != <Vec<String>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .lines
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
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
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = reply_2
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        Ok(response)
    }
    /// Run BatchCreateCertificates after validating the tenant and durably recording the command.
    async fn run_batch_create_certificates(
        &self,
        req: certificates::BatchCreateCertificatesRequest,
    ) -> Result<certificates::BatchCreateCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("BatchCreateCertificates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateCertificates", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(certificates::batch_create_certificates_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = certificates::CheckCertificateEligibilityRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    country_code: (match req.chunk.clone() {
                        Some(certificates::batch_create_certificates_request::Chunk::Record(value)) => value,
                        _ => Default::default(),
                    })
                    .billing_address
                    .clone()
                    .unwrap_or_default()
                    .country_code
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
            let mut client = CertificateValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::CheckCertificateEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_certificate_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = (match req.chunk.clone() {
            Some(certificates::batch_create_certificates_request::Chunk::Record(value)) => value,
            _ => Default::default(),
        })
        .lines
        .clone()
        .into_iter()
        .map(|item| {
            let req = req.clone();
            let channel = self.channel.clone();
            async move {
                let outgoing_1 = ledger::SuggestLedgerAccountsRequest {
                    context: req.context.clone(),
                    filter: Some(ledger::suggest_ledger_accounts_request::Filter {
                        created_after: (match req.chunk.clone() {
                            Some(certificates::batch_create_certificates_request::Chunk::Record(value)) => value,
                            _ => Default::default(),
                        })
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                        labels: item.attributes.clone(),
                        owner_id: item.line_id.clone(),
                        region: (match req.chunk.clone() {
                            Some(certificates::batch_create_certificates_request::Chunk::Record(value)) => value,
                            _ => Default::default(),
                        })
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .region
                        .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                async move {
                    let mut client = LedgerAccountQueryServiceClient::new(channel);
                    let input: tonic::Request<ledger::SuggestLedgerAccountsRequest> =
                        tonic::IntoRequest::into_request(outgoing_1);
                    let context = input.get_ref().context.clone().unwrap_or_default();
                    let input = acme_platform::auth::service_request(input, &context)?;
                    let response = client.suggest_ledger_accounts(input).await?;
                    Ok::<_, tonic::Status>(response.into_inner())
                }
                .await
            }
        })
        .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut command_2 = BatchCreateCertificatesDispatch2::from(req.clone());
        command_2.config_set_children_children_location_lines = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .lines
            .clone();
        let outgoing_2: configuration::CreateConfigSetRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::CreateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = tours::ResolveToursRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_3 = acme_platform::codec::through_any(&outgoing_3, "acme.tours.v1.ResolveToursRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = TourQueryServiceClient::new(channel);
            let input: tonic::Request<tours::ResolveToursRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_tours(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::BatchCreateCertificatesResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        Ok(response)
    }
}
/// The domain command prepared by UpdateCertificate for CheckSearchIndexEligibility.
struct UpdateCertificateDispatch0 {
    context: Option<common::RequestContext>,
    search_index_address_lines: Vec<String>,
    search_index_address_recipient: String,
}
impl From<certificates::UpdateCertificateRequest> for UpdateCertificateDispatch0 {
    fn from(req: certificates::UpdateCertificateRequest) -> Self {
        Self {
            context: req.context.clone(),
            search_index_address_lines: req
                .certificate
                .clone()
                .unwrap_or_default()
                .billing_address
                .clone()
                .unwrap_or_default()
                .lines
                .clone(),
            search_index_address_recipient: req
                .certificate
                .clone()
                .unwrap_or_default()
                .billing_address
                .clone()
                .unwrap_or_default()
                .recipient
                .clone(),
        }
    }
}
impl From<UpdateCertificateDispatch0> for search::CheckSearchIndexEligibilityRequest {
    fn from(value: UpdateCertificateDispatch0) -> Self {
        search::CheckSearchIndexEligibilityRequest {
            context: value.context,
            search_index: Some(search::SearchIndex {
                locator: Some(search::search_index::Locator::Address(common::PostalAddress {
                    lines: value.search_index_address_lines,
                    recipient: value.search_index_address_recipient,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by BatchCreateCertificates for CreateConfigSet.
struct BatchCreateCertificatesDispatch2 {
    context: Option<common::RequestContext>,
    config_set_children_children_location_recipient: String,
    config_set_children_children_audit_created_at: Option<prost_types::Timestamp>,
    config_set_children_children_location_locality: String,
    config_set_children_children_location_lines: Vec<String>,
}
impl From<certificates::BatchCreateCertificatesRequest> for BatchCreateCertificatesDispatch2 {
    fn from(req: certificates::BatchCreateCertificatesRequest) -> Self {
        Self {
            context: req.context.clone(),
            config_set_children_children_location_recipient: (match req.chunk.clone() {
                Some(certificates::batch_create_certificates_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .billing_address
            .clone()
            .unwrap_or_default()
            .recipient
            .clone(),
            config_set_children_children_audit_created_at: (match req.chunk.clone() {
                Some(certificates::batch_create_certificates_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone(),
            config_set_children_children_location_locality: (match req.chunk.clone() {
                Some(certificates::batch_create_certificates_request::Chunk::Record(value)) => value,
                _ => Default::default(),
            })
            .billing_address
            .clone()
            .unwrap_or_default()
            .locality
            .clone(),
            config_set_children_children_location_lines: Default::default(),
        }
    }
}
impl From<BatchCreateCertificatesDispatch2> for configuration::CreateConfigSetRequest {
    fn from(value: BatchCreateCertificatesDispatch2) -> Self {
        configuration::CreateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: value.config_set_children_children_audit_created_at,
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            lines: value.config_set_children_children_location_lines,
                            locality: value.config_set_children_children_location_locality,
                            recipient: value.config_set_children_children_location_recipient,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
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
impl CertificateService for CertificateHandler {
    /// Creates a certificate.
    async fn create_certificate(
        &self,
        request: tonic::Request<certificates::CreateCertificateRequest>,
    ) -> Result<tonic::Response<certificates::CreateCertificateResponse>, tonic::Status> {
        self.run_create_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a certificate.
    async fn get_certificate(
        &self,
        request: tonic::Request<certificates::GetCertificateRequest>,
    ) -> Result<tonic::Response<certificates::GetCertificateResponse>, tonic::Status> {
        self.run_get_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Changes a certificate.
    async fn update_certificate(
        &self,
        request: tonic::Request<certificates::UpdateCertificateRequest>,
    ) -> Result<tonic::Response<certificates::UpdateCertificateResponse>, tonic::Status> {
        self.run_update_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Deletes a certificate.
    async fn delete_certificate(
        &self,
        request: tonic::Request<certificates::DeleteCertificateRequest>,
    ) -> Result<tonic::Response<certificates::DeleteCertificateResponse>, tonic::Status> {
        self.run_delete_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificates a page at a time.
    async fn list_certificates(
        &self,
        request: tonic::Request<certificates::ListCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::ListCertificatesResponse>, tonic::Status> {
        self.run_list_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several certificates at once.
    async fn batch_get_certificates(
        &self,
        request: tonic::Request<certificates::BatchGetCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::BatchGetCertificatesResponse>, tonic::Status> {
        self.run_batch_get_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists certificates a page at a time.
    async fn search_certificates(
        &self,
        request: tonic::Request<certificates::SearchCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::SearchCertificatesResponse>, tonic::Status> {
        self.run_search_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type WatchCertificatesStream =
        tokio_stream::wrappers::ReceiverStream<Result<certificates::WatchCertificatesResponse, tonic::Status>>;
    /// Streams the changes of certificates as they happen.
    async fn watch_certificates(
        &self,
        request: tonic::Request<certificates::WatchCertificatesRequest>,
    ) -> Result<tonic::Response<Self::WatchCertificatesStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_watch_certificates(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Takes a stream of certificates and answers once at the end.
    async fn batch_create_certificates(
        &self,
        request: tonic::Request<tonic::Streaming<certificates::BatchCreateCertificatesRequest>>,
    ) -> Result<tonic::Response<certificates::BatchCreateCertificatesResponse>, tonic::Status> {
        let mut incoming = request.into_inner();
        let mut response = certificates::BatchCreateCertificatesResponse::default();
        while let Some(request) = incoming.message().await? {
            response = self.run_batch_create_certificates(request).await?;
        }
        Ok(tonic::Response::new(response))
    }
}
