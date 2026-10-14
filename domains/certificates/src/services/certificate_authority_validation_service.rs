use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::permission_check_validation_service_client::PermissionCheckValidationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_authority_validation_service_server::CertificateAuthorityValidationService;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ledger::v1::ledger_account_query_service_client::LedgerAccountQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_service_client::SearchIndexServiceClient;
use acme_proto::acme::tours::v1 as tours;
use acme_proto::acme::tours::v1::tour_service_client::TourServiceClient;

/// Checks certificate authorities against the rules before anything acts on them.
#[derive(Clone)]
pub struct CertificateAuthorityValidationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    lint_certificate_authorities_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<certificates::CalculateCsrRequest, certificates::CalculateCsrResponse>,
    >,
    preview_certificate_authority_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<tours::CreateTourRequest, tours::CreateTourResponse>,
    >,
    preview_certificate_authority_1_port: acme_platform::ports::Callback<
        certificates::SimulateCertificateAuthorityRequest,
        certificates::SimulateCertificateAuthorityResponse,
    >,
}
impl CertificateAuthorityValidationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<tours::CreateTourRequest, tours::CreateTourResponse> = {
            let client = TourServiceClient::new(channel.clone());
            std::sync::Arc::new(
                move |request: tours::CreateTourRequest| -> acme_platform::ports::RpcFuture<tours::CreateTourResponse> {
                    let mut client = client.clone();
                    Box::pin(async move {
                        let context = request.context.clone().unwrap_or_default();
                        let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
                        Ok(client.create_tour(request).await?.into_inner())
                    })
                },
            )
        };
        let preview_certificate_authority_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            certificates::SimulateCertificateAuthorityRequest,
            certificates::SimulateCertificateAuthorityResponse,
        > = {
            let client = CertificateAuthorityCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::SimulateCertificateAuthorityRequest| -> acme_platform::ports::RpcFuture<certificates::SimulateCertificateAuthorityResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_certificate_authority(request).await?.into_inner())
})
})
        };
        let preview_certificate_authority_1_port = callback;
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
        let lint_certificate_authorities_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            lint_certificate_authorities_1_port,
            preview_certificate_authority_0_port,
            preview_certificate_authority_1_port,
        }
    }
    /// Run ValidateCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_validate_certificate_authority(
        &self,
        req: certificates::ValidateCertificateAuthorityRequest,
    ) -> Result<certificates::ValidateCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ValidateCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CalculateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                assignee: Some(identity::UserRef {
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = featureflags::ListFlagEvaluationConflictsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_evaluation_conflicts_request::Filter {
                created_after: req
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.annotations.clone(),
                owner_id: req
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
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
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = CsrCalculationServiceClient::new(channel_0);
                let input: tonic::Request<certificates::CalculateCsrRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.calculate_csr(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = FlagEvaluationSyncServiceClient::new(channel_1);
                let input: tonic::Request<featureflags::ListFlagEvaluationConflictsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_flag_evaluation_conflicts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = authorization::ExplainPermissionCheckDecisionRequest {
            context: req.context.clone(),
            details: Some(authorization::explain_permission_check_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
                    .email
                    .clone(),
                    preferred_channel: reply_0
                        .csr
                        .clone()
                        .unwrap_or_default()
                        .requester
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = PermissionCheckValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::ExplainPermissionCheckDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_permission_check_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = search::BatchCreateSearchIndexesRequest {
            context: req.context.clone(),
            chunk: Some(search::batch_create_search_indexes_request::Chunk::Record(
                search::SearchIndex {
                    locator: Some(search::search_index::Locator::Address(common::PostalAddress {
                        locality: req
                            .certificate_authority
                            .clone()
                            .unwrap_or_default()
                            .location
                            .clone()
                            .unwrap_or_default()
                            .locality
                            .clone(),
                        ..Default::default()
                    })),
                    owner: Some(identity::UserRef {
                        user_id: req
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = SearchIndexServiceClient::new(channel);
            let input: tonic::Request<search::BatchCreateSearchIndexesRequest> =
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
            let response = client.batch_create_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    correlation_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
                    kind: reply_2.permission_check.clone().unwrap_or_default().kind.clone(),
                    subject: Some(featureflags::FlagRolloutRef {
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = certificates::ValidateCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .csr
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2
            .permission_check
            .clone()
            .unwrap_or_default()
            .subject
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
        let contribution: String = reply_2
            .permission_check
            .clone()
            .unwrap_or_default()
            .subject
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
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .children[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
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
    /// Run PreviewCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_preview_certificate_authority(
        &self,
        req: certificates::PreviewCertificateAuthorityRequest,
    ) -> Result<certificates::PreviewCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PreviewCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = tours::CreateTourRequest {
            context: req.context.clone(),
            tour: Some(tours::Tour {
                participants: vec![tours::TourParticipant {
                    contact: Some(common::ContactInfo {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.preview_certificate_authority_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = certificates::SimulateCertificateAuthorityRequest {
            certificate_authority: Some(certificates::CertificateAuthority {
                children: vec![certificates::CertificateAuthority {
                    children: vec![certificates::CertificateAuthority {
                        location: Some(common::PostalAddress {
                            country_code: reply_0
                                .tour
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            locality: reply_0
                                .tour
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.preview_certificate_authority_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let mut response = certificates::PreviewCertificateAuthorityResponse::default();
        let contribution: String = reply_0
            .tour
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .recipient
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
                .recipient = contribution;
        }
        let contribution: i32 = reply_1.certificate_authority.clone().unwrap_or_default().depth.clone();
        if contribution != <i32>::default() {
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
                .depth = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: i32 = req.certificate_authority.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .status = contribution;
        }
        Ok(response)
    }
    /// Run CheckCertificateAuthorityEligibility after validating the tenant and durably recording the command.
    async fn run_check_certificate_authority_eligibility(
        &self,
        req: certificates::CheckCertificateAuthorityEligibilityRequest,
    ) -> Result<certificates::CheckCertificateAuthorityEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckCertificateAuthorityEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckCertificateAuthorityEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let command_0 = CheckCertificateAuthorityEligibilityDispatch0::from(req.clone());
        let outgoing_0: certificates::RecalculateCsrsRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CsrCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::RecalculateCsrsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_csrs(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::QuoteConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                subject: Some(configuration::ConfigSetRef {
                    display_name: (match reply_0.csr.clone().unwrap_or_default().origin.clone() {
                        Some(certificates::csr::Origin::Customer(value)) => value,
                        _ => Default::default(),
                    })
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
            let mut client = ConfigChangeCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::QuoteConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::GetCertificateAuthorityRatesRequest {
            certificate_authority_id: req.certificate_authority_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<certificates::GetCertificateAuthorityRatesRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CertificateAuthorityCalculationServiceClient::new(channel);
                let input: tonic::Request<certificates::GetCertificateAuthorityRatesRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.get_certificate_authority_rates(input).await?;
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
        let mut response = certificates::CheckCertificateAuthorityEligibilityResponse::default();
        let contribution: String = reply_0
            .csr
            .clone()
            .unwrap_or_default()
            .assignee
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_0
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_2
            .certificate_authority
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .user_id
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
                .manager
                .get_or_insert_with(Default::default)
                .user_id = contribution;
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
    /// Run NormalizeCertificateAuthority after validating the tenant and durably recording the command.
    async fn run_normalize_certificate_authority(
        &self,
        req: certificates::NormalizeCertificateAuthorityRequest,
    ) -> Result<certificates::NormalizeCertificateAuthorityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("NormalizeCertificateAuthority", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeCertificateAuthority", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ListFeatureFlagConflictsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_feature_flag_conflicts_request::Filter {
                created_after: req
                    .certificate_authority
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
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
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = certificates::ExplainCsrCalculationRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
                    user_id: req
                        .certificate_authority
                        .clone()
                        .unwrap_or_default()
                        .manager
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = FeatureFlagSyncServiceClient::new(channel_0);
                let input: tonic::Request<featureflags::ListFeatureFlagConflictsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_feature_flag_conflicts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CsrCalculationServiceClient::new(channel_1);
                let input: tonic::Request<certificates::ExplainCsrCalculationRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_csr_calculation(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = certificates::NormalizeCertificateAuthorityResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = (match reply_1.csr.clone().unwrap_or_default().origin.clone() {
            Some(certificates::csr::Origin::Customer(value)) => value,
            _ => Default::default(),
        })
        .user_id
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
                .manager
                .get_or_insert_with(Default::default)
                .user_id = contribution;
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
    /// Run ExplainCertificateAuthorityDecision after validating the tenant and durably recording the command.
    async fn run_explain_certificate_authority_decision(
        &self,
        req: certificates::ExplainCertificateAuthorityDecisionRequest,
    ) -> Result<certificates::ExplainCertificateAuthorityDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainCertificateAuthorityDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainCertificateAuthorityDecision", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = ledger::BrowseLedgerAccountsRequest {
            context: req.context.clone(),
            filter: Some(ledger::browse_ledger_accounts_request::Filter {
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = ratelimiting::CompareQuotaBucketOptionsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::compare_quota_bucket_options_request::Details {
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
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = LedgerAccountQueryServiceClient::new(channel_0);
                let input: tonic::Request<ledger::BrowseLedgerAccountsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.browse_ledger_accounts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = QuotaBucketCalculationServiceClient::new(channel_1);
                let input: tonic::Request<ratelimiting::CompareQuotaBucketOptionsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.compare_quota_bucket_options(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = certificates::ExplainCertificateAuthorityDecisionResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
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
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.certificate_authority.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.certificate_authority.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run LintCertificateAuthorities after validating the tenant and durably recording the command.
    async fn run_lint_certificate_authorities(
        &self,
        req: certificates::LintCertificateAuthoritiesRequest,
    ) -> Result<certificates::LintCertificateAuthoritiesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("LintCertificateAuthorities", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintCertificateAuthorities", "running workflow");
        tracing::debug!(audit_value = ?req.certificate_authority.clone().unwrap_or_default().certificate_authority_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CompareQuotaBucketOptionsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::compare_quota_bucket_options_request::Details {
                destination: Some(common::PostalAddress {
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
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::CompareQuotaBucketOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_quota_bucket_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::CalculateCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
                audit: Some(common::AuditInfo {
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
                comments: vec![certificates::CsrComment {
                    created_at: reply_0
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.lint_certificate_authorities_1_port.as_ref(), outgoing_1).await?;
        let mut response = certificates::LintCertificateAuthoritiesResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_0
            .quota_bucket
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
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.certificate_authority_id.clone();
        if contribution != <String>::default() {
            response
                .certificate_authority
                .get_or_insert_with(Default::default)
                .certificate_authority_id = contribution;
        }
        let contribution: String = req.certificate_authority.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.certificate_authority.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run GetCertificateAuthorityRules after validating the tenant and durably recording the command.
    async fn run_get_certificate_authority_rules(
        &self,
        req: certificates::GetCertificateAuthorityRulesRequest,
    ) -> Result<certificates::GetCertificateAuthorityRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetCertificateAuthorityRules", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCertificateAuthorityRules", "running workflow");
        let mut response = certificates::GetCertificateAuthorityRulesResponse::default();
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
/// The domain command prepared by CheckCertificateAuthorityEligibility for RecalculateCsrs.
struct CheckCertificateAuthorityEligibilityDispatch0 {
    context: Option<common::RequestContext>,
    csr_customer_user_id: String,
    details_destination_locality: String,
    csr_customer_display_name: String,
    csr_customer_email: String,
}
impl From<certificates::CheckCertificateAuthorityEligibilityRequest> for CheckCertificateAuthorityEligibilityDispatch0 {
    fn from(req: certificates::CheckCertificateAuthorityEligibilityRequest) -> Self {
        Self {
            context: req.context.clone(),
            csr_customer_user_id: req
                .certificate_authority
                .clone()
                .unwrap_or_default()
                .manager
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            details_destination_locality: req
                .certificate_authority
                .clone()
                .unwrap_or_default()
                .location
                .clone()
                .unwrap_or_default()
                .locality
                .clone(),
            csr_customer_display_name: req
                .certificate_authority
                .clone()
                .unwrap_or_default()
                .manager
                .clone()
                .unwrap_or_default()
                .display_name
                .clone(),
            csr_customer_email: req
                .details
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
        }
    }
}
impl From<CheckCertificateAuthorityEligibilityDispatch0> for certificates::RecalculateCsrsRequest {
    fn from(value: CheckCertificateAuthorityEligibilityDispatch0) -> Self {
        certificates::RecalculateCsrsRequest {
            context: value.context,
            csr: Some(certificates::Csr {
                origin: Some(certificates::csr::Origin::Customer(identity::UserRef {
                    display_name: value.csr_customer_display_name,
                    email: value.csr_customer_email,
                    user_id: value.csr_customer_user_id,
                    ..Default::default()
                })),
                ..Default::default()
            }),
            details: Some(certificates::recalculate_csrs_request::Details {
                destination: Some(common::PostalAddress {
                    locality: value.details_destination_locality,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CertificateAuthorityValidationService for CertificateAuthorityValidationHandler {
    /// Validates a certificate authority.
    async fn validate_certificate_authority(
        &self,
        request: tonic::Request<certificates::ValidateCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::ValidateCertificateAuthorityResponse>, tonic::Status> {
        self.run_validate_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a certificate authority.
    async fn preview_certificate_authority(
        &self,
        request: tonic::Request<certificates::PreviewCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::PreviewCertificateAuthorityResponse>, tonic::Status> {
        self.run_preview_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a certificate authority.
    async fn check_certificate_authority_eligibility(
        &self,
        request: tonic::Request<certificates::CheckCertificateAuthorityEligibilityRequest>,
    ) -> Result<tonic::Response<certificates::CheckCertificateAuthorityEligibilityResponse>, tonic::Status> {
        self.run_check_certificate_authority_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a certificate authority.
    async fn normalize_certificate_authority(
        &self,
        request: tonic::Request<certificates::NormalizeCertificateAuthorityRequest>,
    ) -> Result<tonic::Response<certificates::NormalizeCertificateAuthorityResponse>, tonic::Status> {
        self.run_normalize_certificate_authority(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a certificate authority.
    async fn explain_certificate_authority_decision(
        &self,
        request: tonic::Request<certificates::ExplainCertificateAuthorityDecisionRequest>,
    ) -> Result<tonic::Response<certificates::ExplainCertificateAuthorityDecisionResponse>, tonic::Status> {
        self.run_explain_certificate_authority_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a certificate authority.
    async fn lint_certificate_authorities(
        &self,
        request: tonic::Request<certificates::LintCertificateAuthoritiesRequest>,
    ) -> Result<tonic::Response<certificates::LintCertificateAuthoritiesResponse>, tonic::Status> {
        self.run_lint_certificate_authorities(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a certificate authority.
    async fn get_certificate_authority_rules(
        &self,
        request: tonic::Request<certificates::GetCertificateAuthorityRulesRequest>,
    ) -> Result<tonic::Response<certificates::GetCertificateAuthorityRulesResponse>, tonic::Status> {
        self.run_get_certificate_authority_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
