use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_query_service_client::AdmissionQueryServiceClient;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_history_service_client::PermissionCheckHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_server::CertificateAuthorityCalculationService;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_calculation_service_client::EnvironmentCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_service_client::MetricSeriesServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_service_client::SearchIndexServiceClient;

/// Computes the figures of a certificate authority, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct CertificateAuthorityCalculationApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    compare_certificate_authority_options_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            certificates::RecalculateCertificatesRequest,
            certificates::RecalculateCertificatesResponse,
        >,
    >,
    estimate_certificate_authority_0_port: acme_platform::ports::Callback<
        certificates::EstimateCertificateRequest,
        certificates::EstimateCertificateResponse,
    >,
    estimate_certificate_authority_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            authorization::RevertPermissionCheckRequest,
            authorization::RevertPermissionCheckResponse,
        >,
    >,
    get_certificate_authority_rates_0_port:
        acme_platform::ports::Callback<configuration::QuoteEnvironmentRequest, configuration::QuoteEnvironmentResponse>,
}
impl CertificateAuthorityCalculationApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
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
        let estimate_certificate_authority_0_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::RevertPermissionCheckRequest,
            authorization::RevertPermissionCheckResponse,
        > = {
            let client = PermissionCheckHistoryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::RevertPermissionCheckRequest| -> acme_platform::ports::RpcFuture<authorization::RevertPermissionCheckResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.revert_permission_check(request).await?.into_inner())
})
})
        };
        let estimate_certificate_authority_1_port =
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
        let compare_certificate_authority_options_2_port =
            std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::QuoteEnvironmentRequest,
            configuration::QuoteEnvironmentResponse,
        > = {
            let client = EnvironmentCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::QuoteEnvironmentRequest| -> acme_platform::ports::RpcFuture<configuration::QuoteEnvironmentResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_environment(request).await?.into_inner())
})
})
        };
        let get_certificate_authority_rates_0_port = callback;
        Self {
            channel,
            repository,
            compare_certificate_authority_options_2_port,
            estimate_certificate_authority_0_port,
            estimate_certificate_authority_1_port,
            get_certificate_authority_rates_0_port,
        }
    }
    /// Run CalculateCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_calculate_certificate_authority(
        &self,
        req: certificates::CalculateCertificateAuthorityRequest,
    ) -> Result<certificates::CalculateCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CalculateCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::QuoteCertificateRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
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
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = authorization::AnnotatePermissionCheckVersionRequest {
            context: req.context.clone(),
            permission_check: Some(authorization::PermissionCheck {
                attributes: req.certificate_authority.clone().unwrap_or_default().attributes.clone(),
                subject: Some(authorization::AccessPolicyRef {
                    display_name: req
                        .certificate_authority
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CertificateCalculationServiceClient::new(channel_0);
                let input: tonic::Request<certificates::QuoteCertificateRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.quote_certificate(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = PermissionCheckHistoryServiceClient::new(channel_1);
                let input: tonic::Request<authorization::AnnotatePermissionCheckVersionRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.annotate_permission_check_version(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut command_2 = CalculateCertificateAuthorityDispatch2::from(req.clone());
        command_2.certificate_card_network =
            (match reply_0.certificate.clone().unwrap_or_default().settlement.clone() {
                Some(certificates::certificate::Settlement::Card(value)) => value,
                _ => Default::default(),
            })
            .network
            .clone();
        command_2.details_destination_country_code = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        let outgoing_2: certificates::CompareCertificateOptionsRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CompareCertificateOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_certificate_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = certificates::EstimateCertificateRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
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
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::estimate_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    locality: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
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
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.certificates.v1.EstimateCertificateRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::EstimateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = metrics::BatchCreateMetricSeriesesRequest {
            context: req.context.clone(),
            chunk: Some(metrics::batch_create_metric_serieses_request::Chunk::Record(
                metrics::MetricSeries {
                    locator: Some(metrics::metric_series::Locator::Address(common::PostalAddress {
                        locality: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .destination
                            .clone()
                            .unwrap_or_default()
                            .locality
                            .clone(),
                        region: req
                            .certificate_authority
                            .clone()
                            .unwrap_or_default()
                            .location
                            .clone()
                            .unwrap_or_default()
                            .region
                            .clone(),
                        ..Default::default()
                    })),
                    audit: Some(common::AuditInfo {
                        etag: reply_2
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .etag
                            .clone(),
                        updated_by: req
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = MetricSeriesServiceClient::new(channel);
            let input: tonic::Request<metrics::BatchCreateMetricSeriesesRequest> =
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
            let response = client.batch_create_metric_serieses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    audit: Some(common::AuditInfo {
                        created_by: reply_2
                            .certificate
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
                        postal_code: reply_2
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .billing_address
                            .clone()
                            .unwrap_or_default()
                            .postal_code
                            .clone(),
                        recipient: reply_2
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .billing_address
                            .clone()
                            .unwrap_or_default()
                            .recipient
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
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = certificates::CalculateCertificateAuthorityResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
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
                .certificate_authority
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
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
                .created_by = contribution;
        }
        let contribution: Vec<String> = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
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
        let contribution: String = reply_3
            .certificate
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
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_3
            .certificate
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .display_name
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
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: String = req.certificate_authority.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .parent_id = contribution;
        }
        Ok(response)
    }
    /// Run EstimateCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_estimate_certificate_authority(
        &self,
        req: certificates::EstimateCertificateAuthorityRequest,
    ) -> Result<certificates::EstimateCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("EstimateCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    recipient: req
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
            }),
            context: req.context.clone(),
            details: Some(certificates::estimate_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.estimate_certificate_authority_0_port)(outgoing_0).await?;
        let outgoing_1 = authorization::RevertPermissionCheckRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            permission_check: Some(authorization::PermissionCheck {
                attributes: req.certificate_authority.clone().unwrap_or_default().attributes.clone(),
                correlation_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                subject: Some(authorization::AccessPolicyRef {
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
            }),
            ..Default::default()
        };
        let reply_1 =
            self.estimate_certificate_authority_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = certificates::EstimateCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .region
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
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
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
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run QuoteCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_quote_certificate_authority(
        &self,
        req: certificates::QuoteCertificateAuthorityRequest,
    ) -> Result<certificates::QuoteCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("QuoteCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .certificate_authority
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
            context: req.context.clone(),
            details: Some(certificates::estimate_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    locality: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    postal_code: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    recipient: req
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
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<certificates::EstimateCertificateRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::EstimateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = QuoteCertificateAuthorityDispatch1::from(req.clone());
        command_1.certificate_bank_bic = (match reply_0.certificate.clone().unwrap_or_default().settlement.clone() {
            Some(certificates::certificate::Settlement::Bank(value)) => value,
            _ => Default::default(),
        })
        .bic
        .clone();
        command_1.details_destination_region = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        command_1.certificate_audit_created_by = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        let outgoing_1: certificates::RecalculateCertificatesRequest = command_1.into();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::RecalculateCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::QuoteCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .country_code
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
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: Option<common::Money> = reply_1.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
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
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: Option<identity::UserRef> =
            req.certificate_authority.clone().unwrap_or_default().manager.clone();
        if contribution != <Option<identity::UserRef>>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .manager = contribution;
        }
        Ok(response)
    }
    /// Run CompareCertificateAuthorityOptions after validating the tenant and durably recording the command.
    async fn run_compare_certificate_authority_options(
        &self,
        req: certificates::CompareCertificateAuthorityOptionsRequest,
    ) -> Result<certificates::CompareCertificateAuthorityOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareCertificateAuthorityOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareCertificateAuthorityOptions", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ResolveFeatureFlagConflictRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    updated_at: req
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
                    email: req
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
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = admissions::BrowseAdmissionsRequest {
            context: req.context.clone(),
            filter: Some(admissions::browse_admissions_request::Filter {
                region: req
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = FeatureFlagSyncServiceClient::new(channel_0);
                let input: tonic::Request<featureflags::ResolveFeatureFlagConflictRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.resolve_feature_flag_conflict(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = AdmissionQueryServiceClient::new(channel_1);
                let input: tonic::Request<admissions::BrowseAdmissionsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.browse_admissions(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = certificates::RecalculateCertificatesRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    lines: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::recalculate_certificates_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
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
        let reply_2 =
            acme_platform::ports::execute(self.compare_certificate_authority_options_2_port.as_ref(), outgoing_2)
                .await?;
        let outgoing_3 = certificates::SimulateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: req
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
                billing_address: Some(common::PostalAddress {
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    ..Default::default()
                }),
                lines: vec![certificates::CertificateLine {
                    tax: Some(certificates::CertificateLineTax {
                        amount: (match reply_0.feature_flag.clone().unwrap_or_default().action.clone() {
                            Some(featureflags::feature_flag::Action::Discount(value)) => value,
                            _ => Default::default(),
                        })
                        .amount
                        .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::simulate_certificate_request::Details {
                destination: Some(common::PostalAddress {
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
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::SimulateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::CompareCertificateAuthorityOptionsResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
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
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = reply_0
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
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
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
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
        let contribution: String = reply_2
            .certificate
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
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: Vec<String> = reply_2
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
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
        let contribution: String = reply_3
            .certificate
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_recalculate_certificate_authorities(
        &self,
        req: certificates::RecalculateCertificateAuthoritiesRequest,
    ) -> Result<certificates::RecalculateCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CompareCertificateOptionsRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    recipient: req
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
            }),
            context: req.context.clone(),
            details: Some(certificates::compare_certificate_options_request::Details {
                destination: Some(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    locality: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    region: req
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
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.certificates.v1.CompareCertificateOptionsRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CompareCertificateOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_certificate_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::SimulateCertificateRequest {
            context: req.context.clone(),
            details: Some(certificates::simulate_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    lines: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<certificates::SimulateCertificateRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CertificateCalculationServiceClient::new(channel);
                let input: tonic::Request<certificates::SimulateCertificateRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_certificate(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
            .await
        });
        sender
            .send(outgoing_1)
            .await
            .map_err(|_| tonic::Status::unavailable("worker stopped"))?;
        drop(sender);
        let reply_1 = worker.await.map_err(|_| tonic::Status::internal("worker failed"))??;
        let mut response = certificates::RecalculateCertificateAuthoritiesResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .recipient
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .recipient = contribution;
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
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run SimulateCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_simulate_certificate_authority(
        &self,
        req: certificates::SimulateCertificateAuthorityRequest,
    ) -> Result<certificates::SimulateCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SimulateCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::PreviewConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: req
                        .certificate_authority
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
            context: req.context.clone(),
            details: Some(configuration::preview_config_change_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
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
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = search::BatchGetSearchIndexesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeValidationServiceClient::new(channel_0);
                let input: tonic::Request<configuration::PreviewConfigChangeRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.preview_config_change(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = SearchIndexServiceClient::new(channel_1);
                let input: tonic::Request<search::BatchGetSearchIndexesRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_search_indexes(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = certificates::SimulateCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: Option<common::PostalAddress> =
            req.certificate_authority.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .location = contribution;
        }
        Ok(response)
    }
    /// Run GetCertificateAuthorityRates after validating the tenant and durably recording the command.
    async fn run_get_certificate_authority_rates(
        &self,
        req: certificates::GetCertificateAuthorityRatesRequest,
    ) -> Result<certificates::GetCertificateAuthorityRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetCertificateAuthorityRates", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCertificateAuthorityRates", "running workflow");
        let outgoing_0 = configuration::QuoteEnvironmentRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.get_certificate_authority_rates_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = certificates::GetCertificateAuthorityRatesResponse::default();
        let contribution: String = reply_0
            .environment
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
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_0
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
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
/// The domain command prepared by CalculateCertificateAuthority for CompareCertificateOptions.
struct CalculateCertificateAuthorityDispatch2 {
    context: Option<common::RequestContext>,
    certificate_card_network: String,
    details_destination_country_code: String,
}
impl From<certificates::CalculateCertificateAuthorityRequest> for CalculateCertificateAuthorityDispatch2 {
    fn from(req: certificates::CalculateCertificateAuthorityRequest) -> Self {
        Self {
            context: req.context.clone(),
            certificate_card_network: Default::default(),
            details_destination_country_code: Default::default(),
        }
    }
}
impl From<CalculateCertificateAuthorityDispatch2> for certificates::CompareCertificateOptionsRequest {
    fn from(value: CalculateCertificateAuthorityDispatch2) -> Self {
        certificates::CompareCertificateOptionsRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        network: value.certificate_card_network,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            context: value.context,
            details: Some(certificates::compare_certificate_options_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: value.details_destination_country_code,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by QuoteCertificateAuthority for RecalculateCertificates.
struct QuoteCertificateAuthorityDispatch1 {
    context: Option<common::RequestContext>,
    certificate_bank_bic: String,
    certificate_billing_address_locality: String,
    details_destination_region: String,
    certificate_audit_created_by: String,
}
impl From<certificates::QuoteCertificateAuthorityRequest> for QuoteCertificateAuthorityDispatch1 {
    fn from(req: certificates::QuoteCertificateAuthorityRequest) -> Self {
        Self {
            context: req.context.clone(),
            certificate_bank_bic: Default::default(),
            certificate_billing_address_locality: req
                .details
                .clone()
                .unwrap_or_default()
                .destination
                .clone()
                .unwrap_or_default()
                .locality
                .clone(),
            details_destination_region: Default::default(),
            certificate_audit_created_by: Default::default(),
        }
    }
}
impl From<QuoteCertificateAuthorityDispatch1> for certificates::RecalculateCertificatesRequest {
    fn from(value: QuoteCertificateAuthorityDispatch1) -> Self {
        certificates::RecalculateCertificatesRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    created_by: value.certificate_audit_created_by,
                    ..Default::default()
                }),
                settlement: Some(certificates::certificate::Settlement::Bank(
                    certificates::CertificateBankSettlement {
                        bic: value.certificate_bank_bic,
                        ..Default::default()
                    },
                )),
                billing_address: Some(common::PostalAddress {
                    locality: value.certificate_billing_address_locality,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            details: Some(certificates::recalculate_certificates_request::Details {
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
impl CertificateAuthorityCalculationService for CertificateAuthorityCalculationApi {
    /// Calculates a certificate authority.
    async fn calculate_certificate_authority(
        &self,
        request: tonic::Request<certificates::CalculateCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::CalculateCertificateAuthorityResponse>, tonic::Status> {
        self.run_calculate_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a certificate authority.
    async fn estimate_certificate_authority(
        &self,
        request: tonic::Request<certificates::EstimateCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::EstimateCertificateAuthorityResponse>, tonic::Status> {
        self.run_estimate_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a certificate authority.
    async fn quote_certificate_authority(
        &self,
        request: tonic::Request<certificates::QuoteCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::QuoteCertificateAuthorityResponse>, tonic::Status> {
        self.run_quote_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a certificate authority.
    async fn compare_certificate_authority_options(
        &self,
        request: tonic::Request<certificates::CompareCertificateAuthorityOptionsRequest>,
    ) -> Result<tonic::Response<certificates::CompareCertificateAuthorityOptionsResponse>, tonic::Status> {
        self.run_compare_certificate_authority_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a certificate authority.
    async fn recalculate_certificate_authorities(
        &self,
        request: tonic::Request<certificates::RecalculateCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::RecalculateCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_recalculate_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a certificate authority.
    async fn simulate_certificate_authority(
        &self,
        request: tonic::Request<certificates::SimulateCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::SimulateCertificateAuthorityResponse>, tonic::Status> {
        self.run_simulate_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a certificate authority.
    async fn get_certificate_authority_rates(
        &self,
        request: tonic::Request<certificates::GetCertificateAuthorityRatesRequest>,
    ) -> Result<tonic::Response<certificates::GetCertificateAuthorityRatesResponse>, tonic::Status> {
        self.run_get_certificate_authority_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
