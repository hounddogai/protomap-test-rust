use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::careplans::v1::care_plan_query_service_client::CarePlanQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_approval_service_client::CertificateApprovalServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_approval_service_client::ConfigChangeApprovalServiceClient;
use acme_proto::acme::configuration::v1::environment_workflow_service_client::EnvironmentWorkflowServiceClient;
use acme_proto::acme::datasets::v1 as datasets;
use acme_proto::acme::datasets::v1::dataset_service_client::DatasetServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_query_service_client::FlagRolloutQueryServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_approval_service_client::KeyRingApprovalServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_approval_service_server::KeyRotationApprovalService;
use acme_proto::acme::keymanagement::v1::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_reporting_service_client::KeyRotationReportingServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_approval_service_client::RateLimitApprovalServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Routes key rotations through approval, with escalation and delegation.
#[derive(Clone)]
pub struct KeyRotationApprovalApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
}
impl KeyRotationApprovalApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SubmitKeyRotation after validating the tenant and durably recording the command.
    async fn run_submit_key_rotation(
        &self,
        req: keymanagement::SubmitKeyRotationRequest,
    ) -> Result<keymanagement::SubmitKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SubmitKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SubmitKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ListPendingCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_pending_crypto_keys_request::Filter {
                created_after: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                region: req
                    .key_rotation
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
        let reply_0 = async move {
            let mut client = CryptoKeyApprovalServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ListPendingCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = venues::CountVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::count_venues_request::Filter {
                created_after: req.details.clone().unwrap_or_default().effective_at.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .organizer
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .key_rotation
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
            let mut client = VenueQueryServiceClient::new(channel);
            let input: tonic::Request<venues::CountVenuesRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::SubmitKeyRotationResponse::default();
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
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: Option<common::TimeWindow> = req.key_rotation.clone().unwrap_or_default().window.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.key_rotation.get_or_insert_with(Default::default).window = contribution;
        }
        Ok(response)
    }
    /// Run ApproveKeyRotation after validating the tenant and durably recording the command.
    async fn run_approve_key_rotation(
        &self,
        req: keymanagement::ApproveKeyRotationRequest,
    ) -> Result<keymanagement::ApproveKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ApproveKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ApproveKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ShareKeyRotationReportRequest {
            context: req.context.clone(),
            details: Some(keymanagement::share_key_rotation_report_request::Details {
                recipient: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationReportingServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ShareKeyRotationReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.share_key_rotation_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::PreviewCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    etag: req
                        .key_rotation
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::PreviewCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = certificates::DelegateCertificateApprovalRequest {
            certificate: Some(certificates::Certificate {
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        token: (match reply_1.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Card(value)) => value,
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
            details: Some(certificates::delegate_certificate_approval_request::Details {
                approver: Some(identity::UserRef {
                    email: reply_1
                        .crypto_key
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateApprovalServiceClient::new(channel);
            let input: tonic::Request<certificates::DelegateCertificateApprovalRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.delegate_certificate_approval(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_3 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_3 = keymanagement::PauseCryptoKeyWorkflowRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            contact: Some(common::ContactInfo {
                                preferred_channel: item.contact.clone().unwrap_or_default().preferred_channel.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyWorkflowServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::PauseCryptoKeyWorkflowRequest> =
                            tonic::IntoRequest::into_request(outgoing_3);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.pause_crypto_key_workflow(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_3).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_3 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::ApproveKeyRotationResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
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
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.key_rotation.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run RejectKeyRotation after validating the tenant and durably recording the command.
    async fn run_reject_key_rotation(
        &self,
        req: keymanagement::RejectKeyRotationRequest,
    ) -> Result<keymanagement::RejectKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RejectKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RejectKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().location.clone().unwrap_or_default().recipient.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::TerminateKeyRotationRequest {
            context: req.context.clone(),
            details: Some(keymanagement::terminate_key_rotation_request::Details {
                new_owner: Some(identity::UserRef {
                    display_name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    user_id: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
                        .clone()
                        .unwrap_or_default()
                        .user_id
                        .clone(),
                    ..Default::default()
                }),
                notify_contact: Some(common::ContactInfo {
                    email: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            key_rotation_id: req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationLifecycleServiceClient::new(channel);
            let input: tonic::Request<keymanagement::TerminateKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.terminate_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ScheduleQuotaBucketReportRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            details: Some(ratelimiting::schedule_quota_bucket_report_request::Details {
                filters: reply_0.result.clone().unwrap_or_default().details.clone(),
                period: req.key_rotation.clone().unwrap_or_default().window.clone(),
                ..Default::default()
            }),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketReportingServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ScheduleQuotaBucketReportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_quota_bucket_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::RejectKeyRotationResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: String = req.key_rotation.clone().unwrap_or_default().title.clone();
        if contribution != <String>::default() {
            response.key_rotation.get_or_insert_with(Default::default).title = contribution;
        }
        Ok(response)
    }
    /// Run EscalateKeyRotation after validating the tenant and durably recording the command.
    async fn run_escalate_key_rotation(
        &self,
        req: keymanagement::EscalateKeyRotationRequest,
    ) -> Result<keymanagement::EscalateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EscalateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EscalateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = datasets::CreateDatasetRequest {
            context: req.context.clone(),
            dataset: Some(datasets::Dataset {
                locator: Some(datasets::dataset::Locator::Address(common::PostalAddress {
                    country_code: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    ..Default::default()
                })),
                audit: Some(common::AuditInfo {
                    created_at: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    display_name: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let reply_0 = async move {
            let mut client = DatasetServiceClient::new(channel);
            let input: tonic::Request<datasets::CreateDatasetRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_dataset(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_1 = configuration::DelegateConfigChangeApprovalRequest {
                        context: req.context.clone(),
                        details: Some(configuration::delegate_config_change_approval_request::Details {
                            approver: Some(identity::UserRef {
                                display_name: reply_0
                                    .dataset
                                    .clone()
                                    .unwrap_or_default()
                                    .owner
                                    .clone()
                                    .unwrap_or_default()
                                    .display_name
                                    .clone(),
                                email: item.contact.clone().unwrap_or_default().email.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = ConfigChangeApprovalServiceClient::new(channel);
                        let input: tonic::Request<configuration::DelegateConfigChangeApprovalRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.delegate_config_change_approval(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::EscalateKeyRotationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = (match reply_0.dataset.clone().unwrap_or_default().locator.clone() {
            Some(datasets::dataset::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .region
        .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: i32 = req.key_rotation.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_rotation.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ListPendingKeyRotations after validating the tenant and durably recording the command.
    async fn run_list_pending_key_rotations(
        &self,
        req: keymanagement::ListPendingKeyRotationsRequest,
    ) -> Result<keymanagement::ListPendingKeyRotationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListPendingKeyRotations", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListPendingKeyRotations", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = careplans::ResolveCarePlansRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CarePlanQueryServiceClient::new(channel);
            let input: tonic::Request<careplans::ResolveCarePlansRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_care_plans(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ListPendingKeyRotationsResponse::default();
        let contribution: String = reply_0.page_info.clone().unwrap_or_default().next_page_token.clone();
        if contribution != <String>::default() {
            response.page_info.get_or_insert_with(Default::default).next_page_token = contribution;
        }
        Ok(response)
    }
    /// Run DelegateKeyRotationApproval after validating the tenant and durably recording the command.
    async fn run_delegate_key_rotation_approval(
        &self,
        req: keymanagement::DelegateKeyRotationApprovalRequest,
    ) -> Result<keymanagement::DelegateKeyRotationApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("DelegateKeyRotationApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DelegateKeyRotationApproval", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::PreviewKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .key_rotation
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
                            lines: req
                                .key_rotation
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::PreviewKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::SubmitQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::submit_quota_bucket_request::Details {
                approver: Some(identity::UserRef {
                    display_name: reply_0
                        .key_ring
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
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .key_ring
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
                    email: reply_0
                        .key_ring
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketApprovalServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SubmitQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.submit_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::DelegateKeyRotationApprovalResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.key_rotation.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run RecallKeyRotation after validating the tenant and durably recording the command.
    async fn run_recall_key_rotation(
        &self,
        req: keymanagement::RecallKeyRotationRequest,
    ) -> Result<keymanagement::RecallKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecallKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecallKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::ScheduleKeyRingExpiryRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        location: Some(common::PostalAddress {
                            locality: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
                                .clone(),
                            region: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .region
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            user_id: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .organizer
                                .clone()
                                .unwrap_or_default()
                                .user_id
                                .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .approver
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingLifecycleServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ScheduleKeyRingExpiryRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.schedule_key_ring_expiry(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::BrowseFlagRolloutsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::browse_flag_rollouts_request::Filter {
                created_after: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: reply_0
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .key_rotation
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
            let mut client = FlagRolloutQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::BrowseFlagRolloutsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.browse_flag_rollouts(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_2 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_2 = careplans::FindCarePlansRequest {
                        context: req.context.clone(),
                        filter: Some(careplans::find_care_plans_request::Filter {
                            created_after: reply_0
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            owner_id: item.participant_id.clone(),
                            region: req
                                .key_rotation
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
                    async move {
                        let mut client = CarePlanQueryServiceClient::new(channel);
                        let input: tonic::Request<careplans::FindCarePlansRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.find_care_plans(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let outgoing_3 = keymanagement::SubmitKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .key_rotation
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
                            country_code: req
                                .key_rotation
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .country_code
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: reply_0
                                .key_ring
                                .clone()
                                .unwrap_or_default()
                                .manager
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: reply_0
                                .key_ring
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = KeyRingApprovalServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SubmitKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.submit_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::RecallKeyRotationResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .locality
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .locality = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: i32 = req.key_rotation.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.key_rotation.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run GetKeyRotationApprovalStatus after validating the tenant and durably recording the command.
    async fn run_get_key_rotation_approval_status(
        &self,
        req: keymanagement::GetKeyRotationApprovalStatusRequest,
    ) -> Result<keymanagement::GetKeyRotationApprovalStatusResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetKeyRotationApprovalStatus", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetKeyRotationApprovalStatus", "running workflow");
        let outgoing_0 = keymanagement::ListPendingKeyRingsRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::list_pending_key_rings_request::Filter {
                created_after: req.context.clone().unwrap_or_default().deadline.clone(),
                owner_id: req.key_rotation_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingApprovalServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ListPendingKeyRingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_pending_key_rings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::RecallKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        children: vec![keymanagement::KeyRing {
                            attributes: reply_0.result.clone().unwrap_or_default().details.clone(),
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = KeyRingApprovalServiceClient::new(channel);
            let input: tonic::Request<keymanagement::RecallKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recall_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::GetKeyRotationApprovalStatusResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        Ok(response)
    }
    /// Run CommentKeyRotationApproval after validating the tenant and durably recording the command.
    async fn run_comment_key_rotation_approval(
        &self,
        req: keymanagement::CommentKeyRotationApprovalRequest,
    ) -> Result<keymanagement::CommentKeyRotationApprovalResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CommentKeyRotationApproval", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CommentKeyRotationApproval", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CancelEnvironmentWorkflowRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .approver
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    user_id: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
            let mut client = EnvironmentWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::CancelEnvironmentWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.cancel_environment_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let jobs_1 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_1 = ratelimiting::RecallRateLimitRequest {
                        context: req.context.clone(),
                        details: Some(ratelimiting::recall_rate_limit_request::Details {
                            approver: Some(identity::UserRef {
                                email: item.contact.clone().unwrap_or_default().email.clone(),
                                user_id: reply_0
                                    .environment
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
                        rate_limit: Some(ratelimiting::RateLimit {
                            audit: Some(common::AuditInfo {
                                created_by: req
                                    .key_rotation
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_by
                                    .clone(),
                                updated_at: req
                                    .key_rotation
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
                    async move {
                        let mut client = RateLimitApprovalServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::RecallRateLimitRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.recall_rate_limit(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let jobs_2 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                let reply_0 = reply_0.clone();
                async move {
                    let outgoing_2 = keymanagement::ListKeyRotationReportsRequest {
                        context: req.context.clone(),
                        filter: Some(keymanagement::list_key_rotation_reports_request::Filter {
                            owner_id: item.participant_id.clone(),
                            region: (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .region
                            .clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = KeyRotationReportingServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::ListKeyRotationReportsRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.list_key_rotation_reports(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let jobs_3 = req
            .key_rotation
            .clone()
            .unwrap_or_default()
            .participants
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_3 = keymanagement::CancelCryptoKeyWorkflowRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            contact: Some(common::ContactInfo {
                                name: item.contact.clone().unwrap_or_default().name.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        details: Some(keymanagement::cancel_crypto_key_workflow_request::Details {
                            notify: Some(common::ContactInfo {
                                email: item.contact.clone().unwrap_or_default().email.clone(),
                                phone: item.contact.clone().unwrap_or_default().phone.clone(),
                                preferred_channel: item.contact.clone().unwrap_or_default().preferred_channel.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyWorkflowServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::CancelCryptoKeyWorkflowRequest> =
                            tonic::IntoRequest::into_request(outgoing_3);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.cancel_crypto_key_workflow(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_3).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_3 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::CommentKeyRotationApprovalResponse::default();
        let contribution: String = (match reply_0.environment.clone().unwrap_or_default().locator.clone() {
            Some(configuration::environment::Locator::Address(value)) => value,
            _ => Default::default(),
        })
        .postal_code
        .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = req.key_rotation_id.clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .key_rotation_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.key_rotation.clone().unwrap_or_default().metadata.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .metadata
                .extend(contribution);
        }
        Ok(response)
    }
}
acme_platform::endpoints! { KeyRotationApprovalService, KeyRotationApprovalApi;
unary {
submit_key_rotation (keymanagement::SubmitKeyRotationRequest) -> keymanagement::SubmitKeyRotationResponse => run_submit_key_rotation;
approve_key_rotation (keymanagement::ApproveKeyRotationRequest) -> keymanagement::ApproveKeyRotationResponse => run_approve_key_rotation;
reject_key_rotation (keymanagement::RejectKeyRotationRequest) -> keymanagement::RejectKeyRotationResponse => run_reject_key_rotation;
escalate_key_rotation (keymanagement::EscalateKeyRotationRequest) -> keymanagement::EscalateKeyRotationResponse => run_escalate_key_rotation;
list_pending_key_rotations (keymanagement::ListPendingKeyRotationsRequest) -> keymanagement::ListPendingKeyRotationsResponse => run_list_pending_key_rotations;
delegate_key_rotation_approval (keymanagement::DelegateKeyRotationApprovalRequest) -> keymanagement::DelegateKeyRotationApprovalResponse => run_delegate_key_rotation_approval;
recall_key_rotation (keymanagement::RecallKeyRotationRequest) -> keymanagement::RecallKeyRotationResponse => run_recall_key_rotation;
get_key_rotation_approval_status (keymanagement::GetKeyRotationApprovalStatusRequest) -> keymanagement::GetKeyRotationApprovalStatusResponse => run_get_key_rotation_approval_status;
comment_key_rotation_approval (keymanagement::CommentKeyRotationApprovalRequest) -> keymanagement::CommentKeyRotationApprovalResponse => run_comment_key_rotation_approval;
}
server {
}
client {
}
bidi {
}
}
