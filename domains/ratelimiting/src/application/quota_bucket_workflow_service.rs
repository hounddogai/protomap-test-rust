use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::admissions::v1::admission_calculation_service_client::AdmissionCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_change_reporting_service_client::ConfigChangeReportingServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::configuration::v1::environment_workflow_service_client::EnvironmentWorkflowServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::fleet::v1::vehicle_service_client::VehicleServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_approval_service_client::QuotaBucketApprovalServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_lifecycle_service_client::QuotaBucketLifecycleServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_workflow_service_server::QuotaBucketWorkflowService;
use acme_proto::acme::ratelimiting::v1::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_import_service_client::ThrottleImportServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_reporting_service_client::ThrottleReportingServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_sync_service_client::ThrottleSyncServiceClient;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::sellers::v1 as sellers;
use acme_proto::acme::sellers::v1::seller_service_client::SellerServiceClient;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::sponsorships::v1::sponsor_calculation_service_client::SponsorCalculationServiceClient;
use acme_proto::acme::sponsorships::v1::sponsor_query_service_client::SponsorQueryServiceClient;

/// Runs the long-lived workflow that takes a quota bucket from start to finish.
#[derive(Clone)]
pub struct QuotaBucketWorkflowEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    advance_quota_bucket_workflow_0_port: acme_platform::ports::Callback<
        ratelimiting::ShareThrottleReportRequest,
        ratelimiting::ShareThrottleReportResponse,
    >,
    list_quota_bucket_workflows_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::ListPendingQuotaBucketsRequest,
            ratelimiting::ListPendingQuotaBucketsResponse,
        >,
    >,
    pause_quota_bucket_workflow_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                ratelimiting::ValidateQuotaBucketRequest,
                ratelimiting::ValidateQuotaBucketResponse,
            >,
    >,
    pause_quota_bucket_workflow_1_port: acme_platform::ports::Callback<
        sponsorships::ExplainSponsorCalculationRequest,
        sponsorships::ExplainSponsorCalculationResponse,
    >,
    resume_quota_bucket_workflow_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::InspectRateLimitWorkflowRequest,
            ratelimiting::InspectRateLimitWorkflowResponse,
        >,
    >,
    resume_quota_bucket_workflow_1_port:
        acme_platform::ports::Callback<sponsorships::CountSponsorsRequest, sponsorships::CountSponsorsResponse>,
    start_quota_bucket_workflow_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            configuration::ScheduleConfigChangeExpiryRequest,
            configuration::ScheduleConfigChangeExpiryResponse,
        >,
    >,
}
impl QuotaBucketWorkflowEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::RatelimitingRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            configuration::ScheduleConfigChangeExpiryRequest,
            configuration::ScheduleConfigChangeExpiryResponse,
        > = {
            let client = ConfigChangeLifecycleServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScheduleConfigChangeExpiryRequest| -> acme_platform::ports::RpcFuture<configuration::ScheduleConfigChangeExpiryResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.schedule_config_change_expiry(request).await?.into_inner())
})
})
        };
        let start_quota_bucket_workflow_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            ratelimiting::ShareThrottleReportRequest,
            ratelimiting::ShareThrottleReportResponse,
        > = {
            let client = ThrottleReportingServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ShareThrottleReportRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ShareThrottleReportResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.share_throttle_report(request).await?.into_inner())
})
})
        };
        let advance_quota_bucket_workflow_0_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ValidateQuotaBucketRequest,
            ratelimiting::ValidateQuotaBucketResponse,
        > = {
            let client = QuotaBucketValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ValidateQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ValidateQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.validate_quota_bucket(request).await?.into_inner())
})
})
        };
        let pause_quota_bucket_workflow_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            sponsorships::ExplainSponsorCalculationRequest,
            sponsorships::ExplainSponsorCalculationResponse,
        > = {
            let client = SponsorCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: sponsorships::ExplainSponsorCalculationRequest| -> acme_platform::ports::RpcFuture<sponsorships::ExplainSponsorCalculationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_sponsor_calculation(request).await?.into_inner())
})
})
        };
        let pause_quota_bucket_workflow_1_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::InspectRateLimitWorkflowRequest,
            ratelimiting::InspectRateLimitWorkflowResponse,
        > = {
            let client = RateLimitWorkflowServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::InspectRateLimitWorkflowRequest| -> acme_platform::ports::RpcFuture<ratelimiting::InspectRateLimitWorkflowResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.inspect_rate_limit_workflow(request).await?.into_inner())
})
})
        };
        let resume_quota_bucket_workflow_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            sponsorships::CountSponsorsRequest,
            sponsorships::CountSponsorsResponse,
        > = {
            let client = SponsorQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: sponsorships::CountSponsorsRequest| -> acme_platform::ports::RpcFuture<sponsorships::CountSponsorsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_sponsors(request).await?.into_inner())
})
})
        };
        let resume_quota_bucket_workflow_1_port = callback;
        let callback: acme_platform::ports::Callback<
            ratelimiting::ListPendingQuotaBucketsRequest,
            ratelimiting::ListPendingQuotaBucketsResponse,
        > = {
            let client = QuotaBucketApprovalServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::ListPendingQuotaBucketsRequest| -> acme_platform::ports::RpcFuture<ratelimiting::ListPendingQuotaBucketsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_pending_quota_buckets(request).await?.into_inner())
})
})
        };
        let list_quota_bucket_workflows_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            advance_quota_bucket_workflow_0_port,
            list_quota_bucket_workflows_1_port,
            pause_quota_bucket_workflow_0_port,
            pause_quota_bucket_workflow_1_port,
            resume_quota_bucket_workflow_0_port,
            resume_quota_bucket_workflow_1_port,
            start_quota_bucket_workflow_1_port,
        }
    }
    /// Run StartQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_start_quota_bucket_workflow(
        &self,
        req: ratelimiting::StartQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::StartQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StartQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StartQuotaBucketWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = fleet::CloneVehicleRequest {
            context: req.context.clone(),
            vehicle: Some(fleet::Vehicle {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    updated_at: req
                        .quota_bucket
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = VehicleServiceClient::new(channel);
            let input: tonic::Request<fleet::CloneVehicleRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_vehicle(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ScheduleConfigChangeExpiryRequest {
            context: req.context.clone(),
            details: Some(configuration::schedule_config_change_expiry_request::Details {
                notify: Some(common::ContactInfo {
                    email: reply_0
                        .vehicle
                        .clone()
                        .unwrap_or_default()
                        .owner
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    name: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
            acme_platform::ports::execute(self.start_quota_bucket_workflow_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = featureflags::PushFlagRolloutRequest {
            context: req.context.clone(),
            flag_rollout: Some(featureflags::FlagRollout {
                audit: Some(common::AuditInfo {
                    etag: reply_0
                        .vehicle
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                location: Some(common::PostalAddress {
                    country_code: (match reply_0.vehicle.clone().unwrap_or_default().locator.clone() {
                        Some(fleet::vehicle::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .country_code
                    .clone(),
                    ..Default::default()
                }),
                participants: vec![featureflags::FlagRolloutParticipant {
                    contact: Some(common::ContactInfo {
                        email: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        phone: req
                            .details
                            .clone()
                            .unwrap_or_default()
                            .notify
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
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::PushFlagRolloutRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.push_flag_rollout(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::StartQuotaBucketWorkflowResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_0
            .vehicle
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: i32 = req.quota_bucket.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run AdvanceQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_advance_quota_bucket_workflow(
        &self,
        req: ratelimiting::AdvanceQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::AdvanceQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("AdvanceQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "AdvanceQuotaBucketWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ShareThrottleReportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::share_throttle_report_request::Details {
                recipient: Some(common::ContactInfo {
                    name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let port = acme_platform::ports::opaque(self.advance_quota_bucket_workflow_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = sponsorships::ExplainSponsorCalculationRequest {
            context: req.context.clone(),
            details: Some(sponsorships::explain_sponsor_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    phone: req
                        .quota_bucket
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
            sponsor: Some(sponsorships::Sponsor {
                contact: Some(common::ContactInfo {
                    preferred_channel: req
                        .quota_bucket
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
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<sponsorships::ExplainSponsorCalculationRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = SponsorCalculationServiceClient::new(channel);
            let input: tonic::Request<sponsorships::ExplainSponsorCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_sponsor_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = AdvanceQuotaBucketWorkflowDispatch2::from(req.clone());
        command_2.details_contact_email = reply_1
            .sponsor
            .clone()
            .unwrap_or_default()
            .relationship_manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        command_2.throttle_subject_display_name = reply_0
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        let outgoing_2: ratelimiting::MapThrottleImportColumnsRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ThrottleImportServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::MapThrottleImportColumnsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.map_throttle_import_columns(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::AdvanceQuotaBucketWorkflowResponse::default();
        let contribution: String = reply_0
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .sponsor
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::Money> = req.quota_bucket.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).balance = contribution;
        }
        Ok(response)
    }
    /// Run CancelQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_cancel_quota_bucket_workflow(
        &self,
        req: ratelimiting::CancelQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::CancelQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CancelQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CancelQuotaBucketWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let jobs_0 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_0 = admissions::EstimateAdmissionRequest {
                        admission: Some(admissions::Admission {
                            subject: Some(admissions::GateScanRef {
                                display_name: req
                                    .quota_bucket
                                    .clone()
                                    .unwrap_or_default()
                                    .holder
                                    .clone()
                                    .unwrap_or_default()
                                    .display_name
                                    .clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        annotations: req.annotations.clone(),
                        context: req.context.clone(),
                        details: Some(admissions::estimate_admission_request::Details {
                            budget: item.amount.clone(),
                            currency_code: req.quota_bucket.clone().unwrap_or_default().currency_code.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = AdmissionCalculationServiceClient::new(channel);
                        let input: tonic::Request<admissions::EstimateAdmissionRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.estimate_admission(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let outgoing_1 = sellers::BatchGetSellersRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<sellers::BatchGetSellersRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = SellerServiceClient::new(channel);
                let input: tonic::Request<sellers::BatchGetSellersRequest> = tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_sellers(input).await?;
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
        let outgoing_2 = configuration::SignalEnvironmentWorkflowRequest {
            context: req.context.clone(),
            details: Some(configuration::signal_environment_workflow_request::Details {
                notify: Some(common::ContactInfo {
                    name: req
                        .quota_bucket
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
            }),
            environment: Some(configuration::Environment {
                owner: Some(identity::UserRef {
                    display_name: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .display_name
                        .clone(),
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
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
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = EnvironmentWorkflowServiceClient::new(channel);
            let input: tonic::Request<configuration::SignalEnvironmentWorkflowRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.signal_environment_workflow(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let jobs_3 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_3 = ratelimiting::SubmitQuotaBucketRequest {
                        context: req.context.clone(),
                        quota_bucket: Some(ratelimiting::QuotaBucket {
                            limits: vec![ratelimiting::QuotaBucketLimit {
                                amount: item.amount.clone(),
                                name: item.name.clone(),
                                period: item.period.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = QuotaBucketApprovalServiceClient::new(channel);
                        let input: tonic::Request<ratelimiting::SubmitQuotaBucketRequest> =
                            tonic::IntoRequest::into_request(outgoing_3);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.submit_quota_bucket(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_3).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_3 = replies.into_iter().last().unwrap_or_default();
        let mut response = ratelimiting::CancelQuotaBucketWorkflowResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
            .environment
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.quota_bucket.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run RetryQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_retry_quota_bucket_workflow(
        &self,
        req: ratelimiting::RetryQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::RetryQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RetryQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RetryQuotaBucketWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ReconcileThrottlesRequest {
            context: req.context.clone(),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    version: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.ratelimiting.v1.ReconcileThrottlesRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ThrottleSyncServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ReconcileThrottlesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.reconcile_throttles(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = ratelimiting::RetryQuotaBucketWorkflowResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.outcome.clone().unwrap_or_default().next_steps.clone();
        if contribution != <Vec<String>>::default() {
            response
                .outcome
                .get_or_insert_with(Default::default)
                .next_steps
                .extend(contribution);
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<common::Money> = req.quota_bucket.clone().unwrap_or_default().available.clone();
        if contribution != <Option<common::Money>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).available = contribution;
        }
        Ok(response)
    }
    /// Run PauseQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_pause_quota_bucket_workflow(
        &self,
        req: ratelimiting::PauseQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::PauseQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("PauseQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PauseQuotaBucketWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ValidateQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::validate_quota_bucket_request::Details {
                contact: Some(common::ContactInfo {
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    email: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    phone: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .quota_bucket
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
        let reply_0 =
            acme_platform::ports::execute(self.pause_quota_bucket_workflow_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = sponsorships::ExplainSponsorCalculationRequest {
            context: req.context.clone(),
            details: Some(sponsorships::explain_sponsor_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .email
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            sponsor: Some(sponsorships::Sponsor {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .quota_bucket
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
            ..Default::default()
        };
        let reply_1 = (self.pause_quota_bucket_workflow_1_port)(outgoing_1).await?;
        let mut response = ratelimiting::PauseQuotaBucketWorkflowResponse::default();
        let contribution: Option<common::ContactInfo> =
            reply_0.quota_bucket.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).contact = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .sponsor
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: Option<ratelimiting::RateLimitRef> =
            req.quota_bucket.clone().unwrap_or_default().holder.clone();
        if contribution != <Option<ratelimiting::RateLimitRef>>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).holder = contribution;
        }
        Ok(response)
    }
    /// Run ResumeQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_resume_quota_bucket_workflow(
        &self,
        req: ratelimiting::ResumeQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::ResumeQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResumeQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResumeQuotaBucketWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::InspectRateLimitWorkflowRequest {
            context: req.context.clone(),
            rate_limit_id: req
                .quota_bucket
                .clone()
                .unwrap_or_default()
                .holder
                .clone()
                .unwrap_or_default()
                .rate_limit_id
                .clone(),
            ..Default::default()
        };
        let reply_0 =
            self.resume_quota_bucket_workflow_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = sponsorships::CountSponsorsRequest {
            context: req.context.clone(),
            filter: Some(sponsorships::count_sponsors_request::Filter {
                created_after: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.quota_bucket.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_0
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
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.resume_quota_bucket_workflow_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = ratelimiting::TerminateQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::terminate_quota_bucket_request::Details {
                new_owner: Some(identity::UserRef {
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
                ..Default::default()
            }),
            quota_bucket_id: req.quota_bucket_id.clone(),
            ..Default::default()
        };
        let outgoing_2 = acme_platform::ports::RequestBuilder::<ratelimiting::TerminateQuotaBucketRequest>::default()
            .map(|draft| *draft = outgoing_2)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketLifecycleServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::TerminateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.terminate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagRolloutNoticesRequest {
            context: req.context.clone(),
            subscriber: Some(common::ContactInfo {
                name: req
                    .quota_bucket
                    .clone()
                    .unwrap_or_default()
                    .contact
                    .clone()
                    .unwrap_or_default()
                    .name
                    .clone(),
                phone: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .phone
                    .clone(),
                preferred_channel: req
                    .details
                    .clone()
                    .unwrap_or_default()
                    .notify
                    .clone()
                    .unwrap_or_default()
                    .preferred_channel
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_3 = async move {
            let mut client = FlagRolloutNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutNoticesRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_flag_rollout_notices(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagRolloutNoticesResponse::default();
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
        let mut response = ratelimiting::ResumeQuotaBucketWorkflowResponse::default();
        let contribution: String = reply_0.rate_limit.clone().unwrap_or_default().rate_limit_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .rate_limit_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_2
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
                .quota_bucket
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: String = req.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run InspectQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_inspect_quota_bucket_workflow(
        &self,
        req: ratelimiting::InspectQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::InspectQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("InspectQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "InspectQuotaBucketWorkflow", "running workflow");
        let mut response = ratelimiting::InspectQuotaBucketWorkflowResponse::default();
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        Ok(response)
    }
    /// Run ListQuotaBucketWorkflows after validating the tenant and durably recording the command.
    async fn run_list_quota_bucket_workflows(
        &self,
        req: ratelimiting::ListQuotaBucketWorkflowsRequest,
    ) -> Result<ratelimiting::ListQuotaBucketWorkflowsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListQuotaBucketWorkflows", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListQuotaBucketWorkflows", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::GenerateConfigChangeReportRequest {
            annotations: req.context.clone().unwrap_or_default().baggage.clone(),
            context: req.context.clone(),
            details: Some(configuration::generate_config_change_report_request::Details {
                filters: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeReportingServiceClient::new(channel);
            let input: tonic::Request<configuration::GenerateConfigChangeReportRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.generate_config_change_report(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::ListPendingQuotaBucketsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::list_pending_quota_buckets_request::Filter {
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 =
            acme_platform::ports::execute(self.list_quota_bucket_workflows_1_port.as_ref(), outgoing_1).await?;
        let mut response = ratelimiting::ListQuotaBucketWorkflowsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: i64 = reply_1.page_info.clone().unwrap_or_default().total_size.clone();
        if contribution != <i64>::default() {
            response.page_info.get_or_insert_with(Default::default).total_size = contribution;
        }
        Ok(response)
    }
    /// Run SignalQuotaBucketWorkflow after validating the tenant and durably recording the command.
    async fn run_signal_quota_bucket_workflow(
        &self,
        req: ratelimiting::SignalQuotaBucketWorkflowRequest,
    ) -> Result<ratelimiting::SignalQuotaBucketWorkflowResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("SignalQuotaBucketWorkflow", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SignalQuotaBucketWorkflow", "running workflow");
        tracing::debug!(audit_value = ?req.quota_bucket.clone().unwrap_or_default().quota_bucket_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ValidateThrottleImportRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::validate_throttle_import_request::Details {
                contact: Some(common::ContactInfo {
                    phone: req
                        .quota_bucket
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
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    version: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .version
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::ValidateThrottleImportRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ThrottleImportServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::ValidateThrottleImportRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.validate_throttle_import(input).await?;
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
        let outgoing_1 = ratelimiting::ExplainThrottleDecisionRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::explain_throttle_decision_request::Details {
                contact: Some(common::ContactInfo {
                    email: req
                        .quota_bucket
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
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .notify
                        .clone()
                        .unwrap_or_default()
                        .preferred_channel
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    rate_limit_id: req
                        .quota_bucket
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .rate_limit_id
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = ThrottleValidationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::ExplainThrottleDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_throttle_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let jobs_2 = req
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_2 = configuration::ResolveEnvironmentConflictRequest {
                        context: req.context.clone(),
                        environment: Some(configuration::Environment {
                            audit: Some(common::AuditInfo {
                                created_by: req
                                    .quota_bucket
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .created_by
                                    .clone(),
                                updated_by: req
                                    .quota_bucket
                                    .clone()
                                    .unwrap_or_default()
                                    .audit
                                    .clone()
                                    .unwrap_or_default()
                                    .updated_by
                                    .clone(),
                                ..Default::default()
                            }),
                            name: item.name.clone(),
                            owner: Some(identity::UserRef {
                                email: req
                                    .details
                                    .clone()
                                    .unwrap_or_default()
                                    .notify
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
                    async move {
                        let mut client = EnvironmentSyncServiceClient::new(channel);
                        let input: tonic::Request<configuration::ResolveEnvironmentConflictRequest> =
                            tonic::IntoRequest::into_request(outgoing_2);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.resolve_environment_conflict(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_2).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_2 = replies.into_iter().last().unwrap_or_default();
        let mut response = ratelimiting::SignalQuotaBucketWorkflowResponse::default();
        let contribution: i64 = reply_0
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: i64 = reply_1
            .throttle
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .version
            .clone();
        if contribution != <i64>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .version = contribution;
        }
        let contribution: String = req.quota_bucket_id.clone();
        if contribution != <String>::default() {
            response
                .quota_bucket
                .get_or_insert_with(Default::default)
                .quota_bucket_id = contribution;
        }
        let contribution: i32 = req.quota_bucket.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.quota_bucket.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by AdvanceQuotaBucketWorkflow for MapThrottleImportColumns.
struct AdvanceQuotaBucketWorkflowDispatch2 {
    context: Option<common::RequestContext>,
    details_contact_email: String,
    details_contact_phone: String,
    throttle_subject_display_name: String,
}
impl From<ratelimiting::AdvanceQuotaBucketWorkflowRequest> for AdvanceQuotaBucketWorkflowDispatch2 {
    fn from(req: ratelimiting::AdvanceQuotaBucketWorkflowRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_contact_email: Default::default(),
            details_contact_phone: req
                .details
                .clone()
                .unwrap_or_default()
                .notify
                .clone()
                .unwrap_or_default()
                .phone
                .clone(),
            throttle_subject_display_name: Default::default(),
        }
    }
}
impl From<AdvanceQuotaBucketWorkflowDispatch2> for ratelimiting::MapThrottleImportColumnsRequest {
    fn from(value: AdvanceQuotaBucketWorkflowDispatch2) -> Self {
        ratelimiting::MapThrottleImportColumnsRequest {
            context: value.context,
            details: Some(ratelimiting::map_throttle_import_columns_request::Details {
                contact: Some(common::ContactInfo {
                    email: value.details_contact_email,
                    phone: value.details_contact_phone,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: value.throttle_subject_display_name,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl QuotaBucketWorkflowService for QuotaBucketWorkflowEndpoint {
    /// Starts a quota bucket.
    async fn start_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::StartQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::StartQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_start_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Advances a quota bucket.
    async fn advance_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::AdvanceQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::AdvanceQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_advance_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Cancels a quota bucket.
    async fn cancel_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::CancelQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::CancelQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_cancel_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Retrys a quota bucket.
    async fn retry_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::RetryQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::RetryQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_retry_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Pauses a quota bucket.
    async fn pause_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::PauseQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::PauseQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_pause_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Resumes a quota bucket.
    async fn resume_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::ResumeQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::ResumeQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_resume_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a quota bucket.
    async fn inspect_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::InspectQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::InspectQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_inspect_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists quota buckets a page at a time.
    async fn list_quota_bucket_workflows(
        &self,
        request: tonic::Request<ratelimiting::ListQuotaBucketWorkflowsRequest>,
    ) -> Result<tonic::Response<ratelimiting::ListQuotaBucketWorkflowsResponse>, tonic::Status> {
        self.run_list_quota_bucket_workflows(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Signals a quota bucket.
    async fn signal_quota_bucket_workflow(
        &self,
        request: tonic::Request<ratelimiting::SignalQuotaBucketWorkflowRequest>,
    ) -> Result<tonic::Response<ratelimiting::SignalQuotaBucketWorkflowResponse>, tonic::Status> {
        self.run_signal_quota_bucket_workflow(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
