use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_client::CertificateValidationServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_server::CsrCalculationService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_service_client::PatientServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_client::QuotaBucketQueryServiceClient;
use acme_proto::acme::tours::v1 as tours;
use acme_proto::acme::tours::v1::tour_query_service_client::TourQueryServiceClient;

/// Computes the figures of a csr, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct CsrCalculationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    estimate_csr_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            certificates::CalculateCertificateAuthorityRequest,
            certificates::CalculateCertificateAuthorityResponse,
        >,
    >,
    explain_csr_calculation_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                certificates::EstimateCertificateRequest,
                certificates::EstimateCertificateResponse,
            >,
    >,
    quote_csr_0_port:
        acme_platform::ports::Callback<certificates::QuoteCertificateRequest, certificates::QuoteCertificateResponse>,
    quote_csr_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            certificates::ExplainCertificateDecisionRequest,
            certificates::ExplainCertificateDecisionResponse,
        >,
    >,
    quote_csr_2_port: acme_platform::ports::Callback<
        configuration::CountConfigChangesRequest,
        configuration::CountConfigChangesResponse,
    >,
}
impl CsrCalculationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::CalculateCertificateAuthorityRequest,
            certificates::CalculateCertificateAuthorityResponse,
        > = {
            let client = CertificateAuthorityCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CalculateCertificateAuthorityRequest| -> acme_platform::ports::RpcFuture<certificates::CalculateCertificateAuthorityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_certificate_authority(request).await?.into_inner())
})
})
        };
        let estimate_csr_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::QuoteCertificateRequest,
            certificates::QuoteCertificateResponse,
        > = {
            let client = CertificateCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::QuoteCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::QuoteCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_certificate(request).await?.into_inner())
})
})
        };
        let quote_csr_0_port = callback;
        let callback: acme_platform::ports::Callback<
            certificates::ExplainCertificateDecisionRequest,
            certificates::ExplainCertificateDecisionResponse,
        > = {
            let client = CertificateValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::ExplainCertificateDecisionRequest| -> acme_platform::ports::RpcFuture<certificates::ExplainCertificateDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_certificate_decision(request).await?.into_inner())
})
})
        };
        let quote_csr_1_port = std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::CountConfigChangesRequest,
            configuration::CountConfigChangesResponse,
        > = {
            let client = ConfigChangeQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CountConfigChangesRequest| -> acme_platform::ports::RpcFuture<configuration::CountConfigChangesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_config_changes(request).await?.into_inner())
})
})
        };
        let quote_csr_2_port = callback;
        let callback: acme_platform::ports::Callback<
            certificates::EstimateCertificateRequest,
            certificates::EstimateCertificateResponse,
        > = {
            let client = CertificateCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::EstimateCertificateRequest| -> acme_platform::ports::RpcFuture<certificates::EstimateCertificateResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_certificate(request).await?.into_inner())
})
})
        };
        let explain_csr_calculation_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            estimate_csr_0_port,
            explain_csr_calculation_0_port,
            quote_csr_0_port,
            quote_csr_1_port,
            quote_csr_2_port,
        }
    }
    /// Run CalculateCsr after validating the tenant and durably recording the command.
    async fn run_calculate_csr(
        &self,
        req: certificates::CalculateCsrRequest,
    ) -> Result<certificates::CalculateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
                            locality: req
                                .details
                                .clone()
                                .unwrap_or_default()
                                .destination
                                .clone()
                                .unwrap_or_default()
                                .locality
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: (match req.csr.clone().unwrap_or_default().origin.clone() {
                                Some(certificates::csr::Origin::Customer(value)) => value,
                                _ => Default::default(),
                            })
                            .display_name
                            .clone(),
                            email: (match req.csr.clone().unwrap_or_default().origin.clone() {
                                Some(certificates::csr::Origin::Customer(value)) => value,
                                _ => Default::default(),
                            })
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
        let outgoing_1 = configuration::WatchConfigSetsRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                email: (match req.csr.clone().unwrap_or_default().origin.clone() {
                    Some(certificates::csr::Origin::Customer(value)) => value,
                    _ => Default::default(),
                })
                .email
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_1 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::WatchConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.watch_config_sets(input).await?;
            let mut frames = response.into_inner();
            let mut last = configuration::WatchConfigSetsResponse::default();
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
        let command_2 = CalculateCsrDispatch2::from(req.clone());
        let outgoing_2: ratelimiting::ResolveQuotaBucketsRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketQueryServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ResolveQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = configuration::UpdateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: reply_0
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: (match req.csr.clone().unwrap_or_default().origin.clone() {
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
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::UpdateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = patients::BatchCreatePatientsRequest {
            context: req.context.clone(),
            chunk: Some(patients::batch_create_patients_request::Chunk::Record(
                patients::Patient {
                    primary_address: Some(common::PostalAddress {
                        locality: reply_3
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .location
                            .clone()
                            .unwrap_or_default()
                            .locality
                            .clone(),
                        recipient: reply_0
                            .certificate_authority
                            .clone()
                            .unwrap_or_default()
                            .location
                            .clone()
                            .unwrap_or_default()
                            .recipient
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = PatientServiceClient::new(channel);
            let input: tonic::Request<patients::BatchCreatePatientsRequest> =
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
            let response = client.batch_create_patients(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        etag: reply_3
                            .config_set
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .etag
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
                    owner: Some(identity::UserRef {
                        user_id: reply_3
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_5 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
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
        let mut response = certificates::CalculateCsrResponse::default();
        let contribution: String = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .user_id = contribution;
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
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3
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
                .csr
                .get_or_insert_with(Default::default)
                .requester
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
            .config_set
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.csr.get_or_insert_with(Default::default).comments.is_empty() {
                response
                    .csr
                    .get_or_insert_with(Default::default)
                    .comments
                    .push(Default::default());
            }
            response.csr.get_or_insert_with(Default::default).comments[0].created_at = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.csr.clone().unwrap_or_default().custom_fields.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .custom_fields
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run EstimateCsr after validating the tenant and durably recording the command.
    async fn run_estimate_csr(
        &self,
        req: certificates::EstimateCsrRequest,
    ) -> Result<certificates::EstimateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CalculateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .assignee
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            user_id: req
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
        let reply_0 = acme_platform::ports::execute(self.estimate_csr_0_port.as_ref(), outgoing_0).await?;
        let mut response = certificates::EstimateCsrResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
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
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<identity::UserRef> = req.csr.clone().unwrap_or_default().assignee.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response.csr.get_or_insert_with(Default::default).assignee = contribution;
        }
        Ok(response)
    }
    /// Run QuoteCsr after validating the tenant and durably recording the command.
    async fn run_quote_csr(
        &self,
        req: certificates::QuoteCsrRequest,
    ) -> Result<certificates::QuoteCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::QuoteCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: req
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
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_0 = (self.quote_csr_0_port)(outgoing_0).await?;
        let outgoing_1 = certificates::ExplainCertificateDecisionRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    country_code: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .country_code
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
            context: req.context.clone(),
            details: Some(certificates::explain_certificate_decision_request::Details {
                contact: Some(common::ContactInfo {
                    name: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
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
        let reply_1 =
            self.quote_csr_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let outgoing_2 = configuration::CountConfigChangesRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_changes_request::Filter {
                created_after: reply_0
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: reply_0
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
        let port = acme_platform::ports::opaque(self.quote_csr_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let mut response = certificates::QuoteCsrResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
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
        let contribution: Vec<String> = reply_2.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.csr.clone().unwrap_or_default().requester.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.csr.get_or_insert_with(Default::default).requester = contribution;
        }
        Ok(response)
    }
    /// Run CompareCsrOptions after validating the tenant and durably recording the command.
    async fn run_compare_csr_options(
        &self,
        req: certificates::CompareCsrOptionsRequest,
    ) -> Result<certificates::CompareCsrOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CompareCsrOptions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareCsrOptions", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
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
                        manager: Some(identity::UserRef {
                            display_name: (match req.csr.clone().unwrap_or_default().origin.clone() {
                                Some(certificates::csr::Origin::Customer(value)) => value,
                                _ => Default::default(),
                            })
                            .display_name
                            .clone(),
                            email: req
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
        let outgoing_1 = certificates::CompareCertificateAuthorityOptionsRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            created_at: req
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
                        location: Some(common::PostalAddress {
                            recipient: reply_0
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .recipient
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: (match req.csr.clone().unwrap_or_default().origin.clone() {
                                Some(certificates::csr::Origin::Customer(value)) => value,
                                _ => Default::default(),
                            })
                            .display_name
                            .clone(),
                            user_id: reply_0
                                .certificate_authority
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CompareCertificateAuthorityOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_certificate_authority_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::CompareCsrOptionsResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Option<common::Money> = reply_1.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.csr.clone().unwrap_or_default().custom_fields.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .custom_fields
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run RecalculateCsrs after validating the tenant and durably recording the command.
    async fn run_recalculate_csrs(
        &self,
        req: certificates::RecalculateCsrsRequest,
    ) -> Result<certificates::RecalculateCsrsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecalculateCsrs", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateCsrs", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CalculateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        audit: Some(common::AuditInfo {
                            updated_by: req
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
                        manager: Some(identity::UserRef {
                            display_name: req
                                .csr
                                .clone()
                                .unwrap_or_default()
                                .assignee
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: req
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
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<certificates::CalculateCertificateAuthorityRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CalculateCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = RecalculateCsrsDispatch1::from(req.clone());
        command_1.certificate_audit_updated_by = reply_0
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        let outgoing_1: certificates::ExplainCertificateCalculationRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::ExplainCertificateCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_certificate_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = tours::ExportToursRequest {
            context: req.context.clone(),
            filter: Some(tours::export_tours_request::Filter {
                created_after: req
                    .csr
                    .clone()
                    .unwrap_or_default()
                    .sla
                    .clone()
                    .unwrap_or_default()
                    .due_at
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
        let _reply_2 = async move {
            let mut client = TourQueryServiceClient::new(channel);
            let input: tonic::Request<tours::ExportToursRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_tours(input).await?;
            let mut frames = response.into_inner();
            let mut last = tours::ExportToursResponse::default();
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
        let mut response = certificates::RecalculateCsrsResponse::default();
        let contribution: String = reply_0
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
                .csr
                .get_or_insert_with(Default::default)
                .assignee
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: String = req.csr.clone().unwrap_or_default().subject.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).subject = contribution;
        }
        Ok(response)
    }
    /// Run SimulateCsr after validating the tenant and durably recording the command.
    async fn run_simulate_csr(
        &self,
        req: certificates::SimulateCsrRequest,
    ) -> Result<certificates::SimulateCsrResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateCsr", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateCsr", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let jobs_0 = req
            .csr
            .clone()
            .unwrap_or_default()
            .comments
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = certificates::SimulateCertificateAuthorityRequest {
                        certificate_authority: Some(certificates::CertificateAuthority {
                            children: vec![certificates::CertificateAuthority {
                                audit: Some(common::AuditInfo {
                                    created_at: item.created_at.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }],
                            manager: Some(identity::UserRef {
                                email: item.author_email.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
                        let input: tonic::Request<certificates::SimulateCertificateAuthorityRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.simulate_certificate_authority(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = certificates::CalculateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        location: Some(common::PostalAddress {
                            postal_code: reply_0
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .postal_code
                                .clone(),
                            recipient: reply_0
                                .certificate_authority
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .recipient
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
                        manager: Some(identity::UserRef {
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
            let input: tonic::Request<certificates::CalculateCertificateAuthorityRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_certificate_authority(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = certificates::SimulateCsrResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_1
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: i32 = req.csr.clone().unwrap_or_default().priority.clone();
        if contribution != <i32>::default() {
            response.csr.get_or_insert_with(Default::default).priority = contribution;
        }
        Ok(response)
    }
    /// Run ExplainCsrCalculation after validating the tenant and durably recording the command.
    async fn run_explain_csr_calculation(
        &self,
        req: certificates::ExplainCsrCalculationRequest,
    ) -> Result<certificates::ExplainCsrCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExplainCsrCalculation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainCsrCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.csr.clone().unwrap_or_default().csr_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    etag: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                counterparty: Some(certificates::CsrRef {
                    display_name: req
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .assignee
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
        let reply_0 = acme_platform::ports::execute(self.explain_csr_calculation_0_port.as_ref(), outgoing_0).await?;
        let mut response = certificates::ExplainCsrCalculationResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.certificate.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .csr
                .get_or_insert_with(Default::default)
                .sla
                .get_or_insert_with(Default::default)
                .due_at = contribution;
        }
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        let contribution: i32 = req.csr.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.csr.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run GetCsrRates after validating the tenant and durably recording the command.
    async fn run_get_csr_rates(
        &self,
        req: certificates::GetCsrRatesRequest,
    ) -> Result<certificates::GetCsrRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCsrRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCsrRates", "running workflow");
        let mut response = certificates::GetCsrRatesResponse::default();
        let contribution: String = req.csr_id.clone();
        if contribution != <String>::default() {
            response.csr.get_or_insert_with(Default::default).csr_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by CalculateCsr for ResolveQuotaBuckets.
struct CalculateCsrDispatch2 {
    context: Option<common::RequestContext>,
}
impl From<certificates::CalculateCsrRequest> for CalculateCsrDispatch2 {
    fn from(req: certificates::CalculateCsrRequest) -> Self {
        Self {
            context: req.context.clone(),
        }
    }
}
impl From<CalculateCsrDispatch2> for ratelimiting::ResolveQuotaBucketsRequest {
    fn from(value: CalculateCsrDispatch2) -> Self {
        ratelimiting::ResolveQuotaBucketsRequest {
            context: value.context,
            ..Default::default()
        }
    }
}
/// The domain command prepared by RecalculateCsrs for ExplainCertificateCalculation.
struct RecalculateCsrsDispatch1 {
    context: Option<common::RequestContext>,
    certificate_audit_updated_by: String,
}
impl From<certificates::RecalculateCsrsRequest> for RecalculateCsrsDispatch1 {
    fn from(req: certificates::RecalculateCsrsRequest) -> Self {
        Self {
            context: req.context.clone(),
            certificate_audit_updated_by: Default::default(),
        }
    }
}
impl From<RecalculateCsrsDispatch1> for certificates::ExplainCertificateCalculationRequest {
    fn from(value: RecalculateCsrsDispatch1) -> Self {
        certificates::ExplainCertificateCalculationRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    updated_by: value.certificate_audit_updated_by,
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
impl CsrCalculationService for CsrCalculationHandler {
    /// Calculates a csr.
    async fn calculate_csr(
        &self,
        request: tonic::Request<certificates::CalculateCsrRequest>,
    ) -> Result<tonic::Response<certificates::CalculateCsrResponse>, tonic::Status> {
        self.run_calculate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a csr.
    async fn estimate_csr(
        &self,
        request: tonic::Request<certificates::EstimateCsrRequest>,
    ) -> Result<tonic::Response<certificates::EstimateCsrResponse>, tonic::Status> {
        self.run_estimate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a csr.
    async fn quote_csr(
        &self,
        request: tonic::Request<certificates::QuoteCsrRequest>,
    ) -> Result<tonic::Response<certificates::QuoteCsrResponse>, tonic::Status> {
        self.run_quote_csr(request.into_inner()).await.map(tonic::Response::new)
    }
    /// Compares a csr.
    async fn compare_csr_options(
        &self,
        request: tonic::Request<certificates::CompareCsrOptionsRequest>,
    ) -> Result<tonic::Response<certificates::CompareCsrOptionsResponse>, tonic::Status> {
        self.run_compare_csr_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a csr.
    async fn recalculate_csrs(
        &self,
        request: tonic::Request<certificates::RecalculateCsrsRequest>,
    ) -> Result<tonic::Response<certificates::RecalculateCsrsResponse>, tonic::Status> {
        self.run_recalculate_csrs(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a csr.
    async fn simulate_csr(
        &self,
        request: tonic::Request<certificates::SimulateCsrRequest>,
    ) -> Result<tonic::Response<certificates::SimulateCsrResponse>, tonic::Status> {
        self.run_simulate_csr(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a csr.
    async fn explain_csr_calculation(
        &self,
        request: tonic::Request<certificates::ExplainCsrCalculationRequest>,
    ) -> Result<tonic::Response<certificates::ExplainCsrCalculationResponse>, tonic::Status> {
        self.run_explain_csr_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a csr.
    async fn get_csr_rates(
        &self,
        request: tonic::Request<certificates::GetCsrRatesRequest>,
    ) -> Result<tonic::Response<certificates::GetCsrRatesResponse>, tonic::Status> {
        self.run_get_csr_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
