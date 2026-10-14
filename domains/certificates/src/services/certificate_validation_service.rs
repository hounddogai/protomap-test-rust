use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_query_service_client::AccessPolicyQueryServiceClient;
use acme_proto::acme::authorization::v1::permission_check_export_service_client::PermissionCheckExportServiceClient;
use acme_proto::acme::authorization::v1::role_binding_notification_service_client::RoleBindingNotificationServiceClient;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::boxoffice::v1::box_office_sale_service_client::BoxOfficeSaleServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_validation_service_server::CertificateValidationService;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_service_client::FeatureFlagServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_validation_service_client::NotificationValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;

/// Checks certificates against the rules before anything acts on them.
#[derive(Clone)]
pub struct CertificateValidationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    check_certificate_eligibility_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ExplainConfigSetCalculationRequest,
            configuration::ExplainConfigSetCalculationResponse,
        >,
    >,
    check_certificate_eligibility_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<featureflags::CloneFeatureFlagRequest, featureflags::CloneFeatureFlagResponse>,
    >,
    explain_certificate_decision_0_port: acme_platform::ports::Callback<
        certificates::EstimateCertificateRequest,
        certificates::EstimateCertificateResponse,
    >,
    explain_certificate_decision_2_port: acme_platform::ports::Callback<
        authorization::SharePermissionCheckExportRequest,
        authorization::SharePermissionCheckExportResponse,
    >,
    preview_certificate_3_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            certificates::CompareCertificateOptionsRequest,
            certificates::CompareCertificateOptionsResponse,
        >,
    >,
    score_certificate_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                authorization::BrowseAccessPoliciesRequest,
                authorization::BrowseAccessPoliciesResponse,
            >,
    >,
    score_certificate_1_port: acme_platform::ports::Callback<
        certificates::GetCertificateRatesRequest,
        certificates::GetCertificateRatesResponse,
    >,
    score_certificate_2_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            authorization::UnsubscribeRoleBindingUpdatesRequest,
            authorization::UnsubscribeRoleBindingUpdatesResponse,
        >,
    >,
    score_certificate_3_port: acme_platform::ports::Callback<
        certificates::RecalculateCertificateAuthoritiesRequest,
        certificates::RecalculateCertificateAuthoritiesResponse,
    >,
}
impl CertificateValidationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::CertificatesRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            certificates::CompareCertificateOptionsRequest,
            certificates::CompareCertificateOptionsResponse,
        > = {
            let client = CertificateCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::CompareCertificateOptionsRequest| -> acme_platform::ports::RpcFuture<certificates::CompareCertificateOptionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.compare_certificate_options(request).await?.into_inner())
})
})
        };
        let preview_certificate_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            configuration::ExplainConfigSetCalculationRequest,
            configuration::ExplainConfigSetCalculationResponse,
        > = {
            let client = ConfigSetCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ExplainConfigSetCalculationRequest| -> acme_platform::ports::RpcFuture<configuration::ExplainConfigSetCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_config_set_calculation(request).await?.into_inner())
})
})
        };
        let check_certificate_eligibility_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::CloneFeatureFlagRequest,
            featureflags::CloneFeatureFlagResponse,
        > = {
            let client = FeatureFlagServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CloneFeatureFlagRequest| -> acme_platform::ports::RpcFuture<featureflags::CloneFeatureFlagResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.clone_feature_flag(request).await?.into_inner())
})
})
        };
        let check_certificate_eligibility_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let explain_certificate_decision_0_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::SharePermissionCheckExportRequest,
            authorization::SharePermissionCheckExportResponse,
        > = {
            let client = PermissionCheckExportServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::SharePermissionCheckExportRequest| -> acme_platform::ports::RpcFuture<authorization::SharePermissionCheckExportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_permission_check_export(request).await?.into_inner())
})
})
        };
        let explain_certificate_decision_2_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::BrowseAccessPoliciesRequest,
            authorization::BrowseAccessPoliciesResponse,
        > = {
            let client = AccessPolicyQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::BrowseAccessPoliciesRequest| -> acme_platform::ports::RpcFuture<authorization::BrowseAccessPoliciesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_access_policies(request).await?.into_inner())
})
})
        };
        let score_certificate_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            certificates::GetCertificateRatesRequest,
            certificates::GetCertificateRatesResponse,
        > = {
            let client = CertificateCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::GetCertificateRatesRequest| -> acme_platform::ports::RpcFuture<certificates::GetCertificateRatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.get_certificate_rates(request).await?.into_inner())
})
})
        };
        let score_certificate_1_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::UnsubscribeRoleBindingUpdatesRequest,
            authorization::UnsubscribeRoleBindingUpdatesResponse,
        > = {
            let client = RoleBindingNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::UnsubscribeRoleBindingUpdatesRequest| -> acme_platform::ports::RpcFuture<authorization::UnsubscribeRoleBindingUpdatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.unsubscribe_role_binding_updates(request).await?.into_inner())
})
})
        };
        let score_certificate_2_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            certificates::RecalculateCertificateAuthoritiesRequest,
            certificates::RecalculateCertificateAuthoritiesResponse,
        > = {
            let client = CertificateAuthorityCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: certificates::RecalculateCertificateAuthoritiesRequest| -> acme_platform::ports::RpcFuture<certificates::RecalculateCertificateAuthoritiesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_certificate_authorities(request).await?.into_inner())
})
})
        };
        let score_certificate_3_port = callback;
        Self {
            channel,
            repository,
            check_certificate_eligibility_1_port,
            check_certificate_eligibility_2_port,
            explain_certificate_decision_0_port,
            explain_certificate_decision_2_port,
            preview_certificate_3_port,
            score_certificate_0_port,
            score_certificate_1_port,
            score_certificate_2_port,
            score_certificate_3_port,
        }
    }
    /// Run ValidateCertificate after validating the tenant and durably recording the command.
    async fn run_validate_certificate(
        &self,
        req: certificates::ValidateCertificateRequest,
    ) -> Result<certificates::ValidateCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::CalculateCertificateRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        last4: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .last4
                        .clone(),
                        ..Default::default()
                    },
                )),
                counterparty: Some(certificates::CsrRef {
                    priority: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
                        .clone()
                        .unwrap_or_default()
                        .priority
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<certificates::CalculateCertificateRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CalculateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_1 = ValidateCertificateDispatch1::from(req.clone());
        command_1.details_destination_region = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        command_1.certificate_billing_address_lines = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .lines
            .clone();
        command_1.details_destination_postal_code = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .postal_code
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
        let outgoing_2 = boxoffice::BatchCreateBoxOfficeSalesRequest {
            context: req.context.clone(),
            chunk: Some(boxoffice::batch_create_box_office_sales_request::Chunk::Record(
                boxoffice::BoxOfficeSale {
                    audit: Some(common::AuditInfo {
                        updated_at: reply_0
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .updated_at
                            .clone(),
                        ..Default::default()
                    }),
                    settlement: Some(boxoffice::box_office_sale::Settlement::Bank(
                        boxoffice::BoxOfficeSaleBankSettlement {
                            account_holder: (match reply_0.certificate.clone().unwrap_or_default().settlement.clone() {
                                Some(certificates::certificate::Settlement::Bank(value)) => value,
                                _ => Default::default(),
                            })
                            .account_holder
                            .clone(),
                            ..Default::default()
                        },
                    )),
                    billing_address: Some(common::PostalAddress {
                        region: reply_1
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
                    counterparty: Some(boxoffice::CashDrawerRef {
                        display_name: reply_1
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
            let mut client = BoxOfficeSaleServiceClient::new(channel);
            let input: tonic::Request<boxoffice::BatchCreateBoxOfficeSalesRequest> =
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
            let response = client.batch_create_box_office_sales(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    audit: Some(common::AuditInfo {
                        created_by: reply_0
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_by
                            .clone(),
                        updated_at: reply_0
                            .certificate
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
                        display_name: req
                            .certificate
                            .clone()
                            .unwrap_or_default()
                            .counterparty
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFeatureFlagChangesRequest> =
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
        let mut response = certificates::ValidateCertificateResponse::default();
        let contribution: Vec<String> = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: String = req.certificate.clone().unwrap_or_default().number.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).number = contribution;
        }
        Ok(response)
    }
    /// Run PreviewCertificate after validating the tenant and durably recording the command.
    async fn run_preview_certificate(
        &self,
        req: certificates::PreviewCertificateRequest,
    ) -> Result<certificates::PreviewCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::QuoteCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    updated_by: req
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
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        token: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .token
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(certificates::quote_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    recipient: req
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = acme_platform::transport::deadline(async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::QuoteCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_1 = configuration::FindConfigChangesRequest {
            context: req.context.clone(),
            filter: Some(configuration::find_config_changes_request::Filter {
                created_after: reply_0
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigChangeQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::FindConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.find_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = notifications::LintNotificationsRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                comments: vec![notifications::NotificationComment {
                    created_at: req
                        .certificate
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = NotificationValidationServiceClient::new(channel);
            let input: tonic::Request<notifications::LintNotificationsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.lint_notifications(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = certificates::CompareCertificateOptionsRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    lines: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    locality: req
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
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        token: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .token
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_3 = acme_platform::ports::execute(self.preview_certificate_3_port.as_ref(), outgoing_3).await?;
        let mut response = certificates::PreviewCertificateResponse::default();
        let contribution: Vec<String> = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
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
        let contribution: i32 = reply_0
            .certificate
            .clone()
            .unwrap_or_default()
            .counterparty
            .clone()
            .unwrap_or_default()
            .priority
            .clone();
        if contribution != <i32>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .priority = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_2.outcome.clone().unwrap_or_default().effective_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.outcome.get_or_insert_with(Default::default).effective_at = contribution;
        }
        let contribution: String = (match reply_2.notification.clone().unwrap_or_default().origin.clone() {
            Some(notifications::notification::Origin::Customer(value)) => value,
            _ => Default::default(),
        })
        .display_name
        .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_3.certificate.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).due_at = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> =
            req.certificate.clone().unwrap_or_default().issued_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).issued_at = contribution;
        }
        Ok(response)
    }
    /// Run CheckCertificateEligibility after validating the tenant and durably recording the command.
    async fn run_check_certificate_eligibility(
        &self,
        req: certificates::CheckCertificateEligibilityRequest,
    ) -> Result<certificates::CheckCertificateEligibilityResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CheckCertificateEligibility", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CheckCertificateEligibility", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::RecalculateQuotaBucketsRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                funding: Some(ratelimiting::quota_bucket::Funding::Bank(
                    ratelimiting::QuotaBucketBankFunding {
                        account_holder: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                            Some(certificates::certificate::Settlement::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .account_holder
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecalculateQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ExplainConfigSetCalculationRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            country_code: req
                                .certificate
                                .clone()
                                .unwrap_or_default()
                                .billing_address
                                .clone()
                                .unwrap_or_default()
                                .country_code
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
                        manager: Some(identity::UserRef {
                            email: reply_0
                                .quota_bucket
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.check_certificate_eligibility_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = featureflags::CloneFeatureFlagRequest {
            context: req.context.clone(),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .quota_bucket
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
        let reply_2 =
            acme_platform::ports::execute(self.check_certificate_eligibility_2_port.as_ref(), outgoing_2).await?;
        let mut response = certificates::CheckCertificateEligibilityResponse::default();
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .certificate
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_2.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<common::Money> = req.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.certificate.get_or_insert_with(Default::default).total = contribution;
        }
        Ok(response)
    }
    /// Run NormalizeCertificate after validating the tenant and durably recording the command.
    async fn run_normalize_certificate(
        &self,
        req: certificates::NormalizeCertificateRequest,
    ) -> Result<certificates::NormalizeCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("NormalizeCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "NormalizeCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let jobs_0 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .lines
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = featureflags::CreateFeatureFlagRequest {
                        context: req.context.clone(),
                        feature_flag: Some(featureflags::FeatureFlag {
                            description: item.description.clone(),
                            action: Some(featureflags::feature_flag::Action::Discount(
                                featureflags::FeatureFlagDiscountAction {
                                    amount: item.tax.clone().unwrap_or_default().amount.clone(),
                                    ..Default::default()
                                },
                            )),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = FeatureFlagServiceClient::new(channel);
                        let input: tonic::Request<featureflags::CreateFeatureFlagRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.create_feature_flag(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = certificates::NormalizeCertificateResponse::default();
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.certificate.clone().unwrap_or_default().due_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).due_at = contribution;
        }
        Ok(response)
    }
    /// Run ExplainCertificateDecision after validating the tenant and durably recording the command.
    async fn run_explain_certificate_decision(
        &self,
        req: certificates::ExplainCertificateDecisionRequest,
    ) -> Result<certificates::ExplainCertificateDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainCertificateDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainCertificateDecision", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::EstimateCertificateRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    recipient: req
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
            }),
            context: req.context.clone(),
            details: Some(certificates::estimate_certificate_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .billing_address
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
        let reply_0 = (self.explain_certificate_decision_0_port)(outgoing_0).await?;
        let jobs_1 = req
            .certificate
            .clone()
            .unwrap_or_default()
            .attachments
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = certificates::CompareCertificateOptionsRequest {
                        certificate: Some(certificates::Certificate {
                            attachments: vec![certificates::CertificateAttachment {
                                checksum: item.checksum.clone(),
                                content_type: item.content_type.clone(),
                                file_name: item.file_name.clone(),
                                uri: item.uri.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CertificateCalculationServiceClient::new(channel);
                        let input: tonic::Request<certificates::CompareCertificateOptionsRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.compare_certificate_options(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = authorization::SharePermissionCheckExportRequest {
            context: req.context.clone(),
            details: Some(authorization::share_permission_check_export_request::Details {
                recipient: Some(common::ContactInfo {
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
        let port = acme_platform::ports::opaque(self.explain_certificate_decision_2_port.clone());
        let reply_2 = acme_platform::ports::execute(&port, outgoing_2).await?;
        let mut response = certificates::ExplainCertificateDecisionResponse::default();
        let contribution: Option<prost_types::Timestamp> =
            reply_0.certificate.clone().unwrap_or_default().issued_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).issued_at = contribution;
        }
        let contribution: String = reply_2
            .permission_check
            .clone()
            .unwrap_or_default()
            .correlation_id
            .clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: Option<certificates::CsrRef> =
            req.certificate.clone().unwrap_or_default().counterparty.clone();
        if contribution != <Option<certificates::CsrRef>>::default() {
            response.certificate.get_or_insert_with(Default::default).counterparty = contribution;
        }
        Ok(response)
    }
    /// Run ScoreCertificate after validating the tenant and durably recording the command.
    async fn run_score_certificate(
        &self,
        req: certificates::ScoreCertificateRequest,
    ) -> Result<certificates::ScoreCertificateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreCertificate", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreCertificate", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().number.clone(), "recorded audit field");
        let outgoing_0 = authorization::BrowseAccessPoliciesRequest {
            context: req.context.clone(),
            filter: Some(authorization::browse_access_policies_request::Filter {
                created_after: req
                    .certificate
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.certificate.clone().unwrap_or_default().metadata.clone(),
                owner_id: req.certificate_id.clone(),
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
        let reply_0 = acme_platform::ports::execute(self.score_certificate_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = certificates::GetCertificateRatesRequest {
            certificate_id: req.certificate.clone().unwrap_or_default().certificate_id.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 = (self.score_certificate_1_port)(outgoing_1).await?;
        let outgoing_2 = authorization::UnsubscribeRoleBindingUpdatesRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let reply_2 =
            self.score_certificate_2_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_2)
            .await?;
        let outgoing_3 = certificates::RecalculateCertificateAuthoritiesRequest {
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
        let port = acme_platform::ports::opaque(self.score_certificate_3_port.clone());
        let reply_3 = acme_platform::ports::execute(&port, outgoing_3).await?;
        let mut response = certificates::ScoreCertificateResponse::default();
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
                .certificate
                .get_or_insert_with(Default::default)
                .billing_address
                .get_or_insert_with(Default::default)
                .country_code = contribution;
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
            response
                .certificate
                .get_or_insert_with(Default::default)
                .counterparty
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2
            .role_binding
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
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
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
        let contribution: Option<common::Money> = req.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.certificate.get_or_insert_with(Default::default).total = contribution;
        }
        Ok(response)
    }
    /// Run LintCertificates after validating the tenant and durably recording the command.
    async fn run_lint_certificates(
        &self,
        req: certificates::LintCertificatesRequest,
    ) -> Result<certificates::LintCertificatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintCertificates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintCertificates", "running workflow");
        tracing::debug!(audit_value = ?req.certificate.clone().unwrap_or_default().certificate_id.clone(), "recorded audit field");
        let command_0 = LintCertificatesDispatch0::from(req.clone());
        let outgoing_0: certificates::ExplainCertificateCalculationRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::ExplainCertificateCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_certificate_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = certificates::CompareCertificateOptionsRequest {
            certificate: Some(certificates::Certificate {
                counterparty: Some(certificates::CsrRef {
                    csr_id: req
                        .certificate
                        .clone()
                        .unwrap_or_default()
                        .counterparty
                        .clone()
                        .unwrap_or_default()
                        .csr_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.certificates.v1.CompareCertificateOptionsRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CompareCertificateOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_certificate_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::LintCertificatesResponse::default();
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: Option<prost_types::Timestamp> =
            reply_1.certificate.clone().unwrap_or_default().issued_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.certificate.get_or_insert_with(Default::default).issued_at = contribution;
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        let contribution: i32 = req.certificate.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.certificate.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run GetCertificateRules after validating the tenant and durably recording the command.
    async fn run_get_certificate_rules(
        &self,
        req: certificates::GetCertificateRulesRequest,
    ) -> Result<certificates::GetCertificateRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCertificateRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCertificateRules", "running workflow");
        let outgoing_0 = configuration::SummarizeConfigChangesRequest {
            context: req.context.clone(),
            details: Some(configuration::summarize_config_changes_request::Details {
                filters: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::ports::RequestBuilder::<configuration::SummarizeConfigChangesRequest>::default()
                .map(|draft| *draft = outgoing_0)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeQueryServiceClient::new(channel);
            let input: tonic::Request<configuration::SummarizeConfigChangesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_config_changes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = certificates::GetCertificateRulesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = req.certificate_id.clone();
        if contribution != <String>::default() {
            response.certificate.get_or_insert_with(Default::default).certificate_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by ValidateCertificate for RecalculateCertificates.
struct ValidateCertificateDispatch1 {
    context: Option<common::RequestContext>,
    certificate_bank_account_holder: String,
    details_destination_region: String,
    certificate_billing_address_lines: Vec<String>,
    details_destination_postal_code: String,
}
impl From<certificates::ValidateCertificateRequest> for ValidateCertificateDispatch1 {
    fn from(req: certificates::ValidateCertificateRequest) -> Self {
        Self {
            context: req.context.clone(),
            certificate_bank_account_holder: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                Some(certificates::certificate::Settlement::Bank(value)) => value,
                _ => Default::default(),
            })
            .account_holder
            .clone(),
            details_destination_region: Default::default(),
            certificate_billing_address_lines: Default::default(),
            details_destination_postal_code: Default::default(),
        }
    }
}
impl From<ValidateCertificateDispatch1> for certificates::RecalculateCertificatesRequest {
    fn from(value: ValidateCertificateDispatch1) -> Self {
        certificates::RecalculateCertificatesRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Bank(
                    certificates::CertificateBankSettlement {
                        account_holder: value.certificate_bank_account_holder,
                        ..Default::default()
                    },
                )),
                billing_address: Some(common::PostalAddress {
                    lines: value.certificate_billing_address_lines,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            details: Some(certificates::recalculate_certificates_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: value.details_destination_postal_code,
                    region: value.details_destination_region,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by LintCertificates for ExplainCertificateCalculation.
struct LintCertificatesDispatch0 {
    context: Option<common::RequestContext>,
    certificate_bank_bic: String,
    certificate_billing_address_recipient: String,
    certificate_bank_account_holder: String,
    details_contact_name: Option<common::PersonName>,
}
impl From<certificates::LintCertificatesRequest> for LintCertificatesDispatch0 {
    fn from(req: certificates::LintCertificatesRequest) -> Self {
        Self {
            context: req.context.clone(),
            certificate_bank_bic: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                Some(certificates::certificate::Settlement::Bank(value)) => value,
                _ => Default::default(),
            })
            .bic
            .clone(),
            certificate_billing_address_recipient: req
                .certificate
                .clone()
                .unwrap_or_default()
                .billing_address
                .clone()
                .unwrap_or_default()
                .recipient
                .clone(),
            certificate_bank_account_holder: (match req.certificate.clone().unwrap_or_default().settlement.clone() {
                Some(certificates::certificate::Settlement::Bank(value)) => value,
                _ => Default::default(),
            })
            .account_holder
            .clone(),
            details_contact_name: req
                .details
                .clone()
                .unwrap_or_default()
                .contact
                .clone()
                .unwrap_or_default()
                .name
                .clone(),
        }
    }
}
impl From<LintCertificatesDispatch0> for certificates::ExplainCertificateCalculationRequest {
    fn from(value: LintCertificatesDispatch0) -> Self {
        certificates::ExplainCertificateCalculationRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Bank(
                    certificates::CertificateBankSettlement {
                        account_holder: value.certificate_bank_account_holder,
                        bic: value.certificate_bank_bic,
                        ..Default::default()
                    },
                )),
                billing_address: Some(common::PostalAddress {
                    recipient: value.certificate_billing_address_recipient,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: value.context,
            details: Some(certificates::explain_certificate_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    name: value.details_contact_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CertificateValidationService for CertificateValidationController {
    /// Validates a certificate.
    async fn validate_certificate(
        &self,
        request: tonic::Request<certificates::ValidateCertificateRequest>,
    ) -> Result<tonic::Response<certificates::ValidateCertificateResponse>, tonic::Status> {
        self.run_validate_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a certificate.
    async fn preview_certificate(
        &self,
        request: tonic::Request<certificates::PreviewCertificateRequest>,
    ) -> Result<tonic::Response<certificates::PreviewCertificateResponse>, tonic::Status> {
        self.run_preview_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Checks a certificate.
    async fn check_certificate_eligibility(
        &self,
        request: tonic::Request<certificates::CheckCertificateEligibilityRequest>,
    ) -> Result<tonic::Response<certificates::CheckCertificateEligibilityResponse>, tonic::Status> {
        self.run_check_certificate_eligibility(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Normalizes a certificate.
    async fn normalize_certificate(
        &self,
        request: tonic::Request<certificates::NormalizeCertificateRequest>,
    ) -> Result<tonic::Response<certificates::NormalizeCertificateResponse>, tonic::Status> {
        self.run_normalize_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a certificate.
    async fn explain_certificate_decision(
        &self,
        request: tonic::Request<certificates::ExplainCertificateDecisionRequest>,
    ) -> Result<tonic::Response<certificates::ExplainCertificateDecisionResponse>, tonic::Status> {
        self.run_explain_certificate_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a certificate.
    async fn score_certificate(
        &self,
        request: tonic::Request<certificates::ScoreCertificateRequest>,
    ) -> Result<tonic::Response<certificates::ScoreCertificateResponse>, tonic::Status> {
        self.run_score_certificate(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a certificate.
    async fn lint_certificates(
        &self,
        request: tonic::Request<certificates::LintCertificatesRequest>,
    ) -> Result<tonic::Response<certificates::LintCertificatesResponse>, tonic::Status> {
        self.run_lint_certificates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a certificate.
    async fn get_certificate_rules(
        &self,
        request: tonic::Request<certificates::GetCertificateRulesRequest>,
    ) -> Result<tonic::Response<certificates::GetCertificateRulesResponse>, tonic::Status> {
        self.run_get_certificate_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
