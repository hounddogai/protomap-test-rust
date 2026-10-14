use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;
use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::careplans::v1::care_plan_calculation_service_client::CarePlanCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_query_service_server::KeyRingQueryService;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_validation_service_client::KeyRotationValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;

/// Answers read-only questions about key rings, for screens and reports.
#[derive(Clone)]
pub struct KeyRingQueryApi {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    browse_key_rings_0_port:
        acme_platform::ports::Callback<ratelimiting::UpdateRateLimitRequest, ratelimiting::UpdateRateLimitResponse>,
    export_key_rings_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            authorization::ListRoleBindingVersionsRequest,
            authorization::ListRoleBindingVersionsResponse,
        >,
    >,
    summarize_key_rings_1_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            ratelimiting::NormalizeQuotaBucketRequest,
            ratelimiting::NormalizeQuotaBucketResponse,
        >,
    >,
}
impl KeyRingQueryApi {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::UpdateRateLimitRequest,
            ratelimiting::UpdateRateLimitResponse,
        > = {
            let client = RateLimitServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::UpdateRateLimitRequest| -> acme_platform::ports::RpcFuture<ratelimiting::UpdateRateLimitResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.update_rate_limit(request).await?.into_inner())
})
})
        };
        let browse_key_rings_0_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::ListRoleBindingVersionsRequest,
            authorization::ListRoleBindingVersionsResponse,
        > = {
            let client = RoleBindingHistoryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::ListRoleBindingVersionsRequest| -> acme_platform::ports::RpcFuture<authorization::ListRoleBindingVersionsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_role_binding_versions(request).await?.into_inner())
})
})
        };
        let export_key_rings_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            ratelimiting::NormalizeQuotaBucketRequest,
            ratelimiting::NormalizeQuotaBucketResponse,
        > = {
            let client = QuotaBucketValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::NormalizeQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::NormalizeQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_quota_bucket(request).await?.into_inner())
})
})
        };
        let summarize_key_rings_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        Self {
            channel,
            repository,
            browse_key_rings_0_port,
            export_key_rings_0_port,
            summarize_key_rings_1_port,
        }
    }
    /// Run LookupKeyRing after validating the tenant and durably recording the command.
    async fn run_lookup_key_ring(
        &self,
        req: keymanagement::LookupKeyRingRequest,
    ) -> Result<keymanagement::LookupKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupKeyRing", "running workflow");
        let command_0 = LookupKeyRingDispatch0::from(req.clone());
        let outgoing_0: keymanagement::QuoteKeyRingRequest = command_0.into();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::QuoteKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::ExplainKeyRingCalculationRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
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
        let outgoing_1 =
            acme_platform::codec::through_any(&outgoing_1, "acme.keymanagement.v1.ExplainKeyRingCalculationRequest")?;
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = KeyRingCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ExplainKeyRingCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_key_ring_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::LookupKeyRingResponse::default();
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_1
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseKeyRings after validating the tenant and durably recording the command.
    async fn run_browse_key_rings(
        &self,
        req: keymanagement::BrowseKeyRingsRequest,
    ) -> Result<keymanagement::BrowseKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::UpdateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
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
                parameters: req.filter.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = (self.browse_key_rings_0_port)(outgoing_0).await?;
        let mut response = keymanagement::BrowseKeyRingsResponse::default();
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
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            response.key_rings[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_0
            .rate_limit
            .clone()
            .unwrap_or_default()
            .owner
            .clone()
            .unwrap_or_default()
            .user_id
            .clone();
        if contribution != <String>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            if response.key_rings[0].children[0].children.is_empty() {
                response.key_rings[0].children[0].children.push(Default::default());
            }
            response.key_rings[0].children[0].children[0]
                .manager
                .get_or_insert_with(Default::default)
                .user_id = contribution;
        }
        Ok(response)
    }
    /// Run FindKeyRings after validating the tenant and durably recording the command.
    async fn run_find_key_rings(
        &self,
        req: keymanagement::FindKeyRingsRequest,
    ) -> Result<keymanagement::FindKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::PreviewKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                location: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                organizer: Some(identity::UserRef {
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
                participants: vec![keymanagement::KeyRotationParticipant {
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let outgoing_0 = acme_platform::ports::RequestBuilder::<keymanagement::PreviewKeyRotationRequest>::default()
            .map(|draft| *draft = outgoing_0)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRotationValidationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::PreviewKeyRotationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.preview_key_rotation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::FindKeyRingsResponse::default();
        let contribution: String = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .organizer
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            response.key_rings[0]
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .key_rotation
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            response.key_rings[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        Ok(response)
    }
    /// Run CountKeyRings after validating the tenant and durably recording the command.
    async fn run_count_key_rings(
        &self,
        req: keymanagement::CountKeyRingsRequest,
    ) -> Result<keymanagement::CountKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = keymanagement::CountKeyRingsResponse::default();
        Ok(response)
    }
    /// Run ResolveKeyRings after validating the tenant and durably recording the command.
    async fn run_resolve_key_rings(
        &self,
        req: keymanagement::ResolveKeyRingsRequest,
    ) -> Result<keymanagement::ResolveKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveKeyRings", "running workflow");
        let outgoing_0 = careplans::CompareCarePlanOptionsRequest {
            care_plan: Some(careplans::CarePlan {
                counterparty: Some(appointments::AppointmentRef {
                    starts_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                metadata: (req.context.clone().unwrap_or_default().baggage.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CarePlanCalculationServiceClient::new(channel);
            let input: tonic::Request<careplans::CompareCarePlanOptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.compare_care_plan_options(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ResolveKeyRingsResponse::default();
        let contribution: String = reply_0
            .care_plan
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            response.key_rings[0].audit.get_or_insert_with(Default::default).etag = contribution;
        }
        let contribution: String = reply_0
            .care_plan
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .country_code
            .clone();
        if contribution != <String>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            if response.key_rings[0].children[0].children.is_empty() {
                response.key_rings[0].children[0].children.push(Default::default());
            }
            response.key_rings[0].children[0].children[0]
                .location
                .get_or_insert_with(Default::default)
                .country_code = contribution;
        }
        Ok(response)
    }
    /// Run ExportKeyRings after validating the tenant and durably recording the command.
    async fn run_export_key_rings(
        &self,
        req: keymanagement::ExportKeyRingsRequest,
    ) -> Result<keymanagement::ExportKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.format.clone(), "recorded audit field");
        let outgoing_0 = authorization::ListRoleBindingVersionsRequest {
            context: req.context.clone(),
            filter: Some(authorization::list_role_binding_versions_request::Filter {
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 =
            self.export_key_rings_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let outgoing_1 = authorization::RecordRoleBindingVersionRequest {
            context: req.context.clone(),
            details: Some(authorization::record_role_binding_version_request::Details {
                author_email: req
                    .context
                    .clone()
                    .unwrap_or_default()
                    .actor
                    .clone()
                    .unwrap_or_default()
                    .email
                    .clone(),
                snapshot: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            role_binding: Some(authorization::RoleBinding {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                lines: vec![authorization::RoleBindingLine {
                    attributes: reply_0.result.clone().unwrap_or_default().details.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = RoleBindingHistoryServiceClient::new(channel);
            let input: tonic::Request<authorization::RecordRoleBindingVersionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.record_role_binding_version(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ExportKeyRingsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1
            .role_binding
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        if contribution != <String>::default() {
            if response
                .key_ring
                .get_or_insert_with(Default::default)
                .children
                .is_empty()
            {
                response
                    .key_ring
                    .get_or_insert_with(Default::default)
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0]
                .location
                .get_or_insert_with(Default::default)
                .postal_code = contribution;
        }
        let contribution: String = req.filter.clone().unwrap_or_default().region.clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        Ok(response)
    }
    /// Run SummarizeKeyRings after validating the tenant and durably recording the command.
    async fn run_summarize_key_rings(
        &self,
        req: keymanagement::SummarizeKeyRingsRequest,
    ) -> Result<keymanagement::SummarizeKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SummarizeKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SummarizeKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.details.clone().unwrap_or_default().format.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::CheckKeyRingEligibilityRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        key_ring_id: req.key_ring_id.clone(),
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
            let input: tonic::Request<keymanagement::CheckKeyRingEligibilityRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.check_key_ring_eligibility(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = ratelimiting::NormalizeQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    created_by: reply_0
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    etag: reply_0
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
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
        let reply_1 = acme_platform::ports::execute(self.summarize_key_rings_1_port.as_ref(), outgoing_1).await?;
        let mut response = keymanagement::SummarizeKeyRingsResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
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
        let contribution: Option<common::TimeWindow> = req.details.clone().unwrap_or_default().period.clone();
        if contribution != <Option<common::TimeWindow>>::default() {
            response.period = contribution;
        }
        Ok(response)
    }
    /// Run SuggestKeyRings after validating the tenant and durably recording the command.
    async fn run_suggest_key_rings(
        &self,
        req: keymanagement::SuggestKeyRingsRequest,
    ) -> Result<keymanagement::SuggestKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::RecalculateKeyRingsRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = KeyRingCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::RecalculateKeyRingsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_key_rings(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::SuggestKeyRingsResponse::default();
        let contribution: Option<common::PostalAddress> = reply_0.key_ring.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            if response.key_rings[0].children[0].children.is_empty() {
                response.key_rings[0].children[0].children.push(Default::default());
            }
            response.key_rings[0].children[0].children[0].location = contribution;
        }
        let contribution: Option<common::PostalAddress> = reply_0.key_ring.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            if response.key_rings.is_empty() {
                response.key_rings.push(Default::default());
            }
            if response.key_rings[0].children.is_empty() {
                response.key_rings[0].children.push(Default::default());
            }
            response.key_rings[0].children[0].location = contribution;
        }
        Ok(response)
    }
    /// Run PeekKeyRing after validating the tenant and durably recording the command.
    async fn run_peek_key_ring(
        &self,
        req: keymanagement::PeekKeyRingRequest,
    ) -> Result<keymanagement::PeekKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PeekKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PeekKeyRing", "running workflow");
        let outgoing_0 = ratelimiting::SearchRateLimitsRequest {
            context: req.context.clone(),
            filter: Some(ratelimiting::search_rate_limits_request::Filter {
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                owner_id: req.key_ring_id.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ratelimiting::SearchRateLimitsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = RateLimitServiceClient::new(channel);
                let input: tonic::Request<ratelimiting::SearchRateLimitsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.search_rate_limits(input).await?;
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
        let mut response = keymanagement::PeekKeyRingResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by LookupKeyRing for QuoteKeyRing.
struct LookupKeyRingDispatch0 {
    context: Option<common::RequestContext>,
    key_ring_children_children_children_key_ring_id: String,
    key_ring_children_manager_email: String,
    key_ring_children_children_manager_user_id: String,
    key_ring_children_children_audit_updated_at: Option<prost_types::Timestamp>,
}
impl From<keymanagement::LookupKeyRingRequest> for LookupKeyRingDispatch0 {
    fn from(req: keymanagement::LookupKeyRingRequest) -> Self {
        Self {
            context: req.context.clone(),
            key_ring_children_children_children_key_ring_id: req.key_ring_id.clone(),
            key_ring_children_manager_email: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .email
                .clone(),
            key_ring_children_children_manager_user_id: req
                .context
                .clone()
                .unwrap_or_default()
                .actor
                .clone()
                .unwrap_or_default()
                .user_id
                .clone(),
            key_ring_children_children_audit_updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
        }
    }
}
impl From<LookupKeyRingDispatch0> for keymanagement::QuoteKeyRingRequest {
    fn from(value: LookupKeyRingDispatch0) -> Self {
        keymanagement::QuoteKeyRingRequest {
            context: value.context,
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            updated_at: value.key_ring_children_children_audit_updated_at,
                            ..Default::default()
                        }),
                        children: vec![keymanagement::KeyRing {
                            key_ring_id: value.key_ring_children_children_children_key_ring_id,
                            ..Default::default()
                        }],
                        manager: Some(identity::UserRef {
                            user_id: value.key_ring_children_children_manager_user_id,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        email: value.key_ring_children_manager_email,
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRingQueryService for KeyRingQueryApi {
    /// Returns a key ring.
    async fn lookup_key_ring(
        &self,
        request: tonic::Request<keymanagement::LookupKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::LookupKeyRingResponse>, tonic::Status> {
        self.run_lookup_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists key rings a page at a time.
    async fn browse_key_rings(
        &self,
        request: tonic::Request<keymanagement::BrowseKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::BrowseKeyRingsResponse>, tonic::Status> {
        self.run_browse_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists key rings a page at a time.
    async fn find_key_rings(
        &self,
        request: tonic::Request<keymanagement::FindKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::FindKeyRingsResponse>, tonic::Status> {
        self.run_find_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts key rings by status.
    async fn count_key_rings(
        &self,
        request: tonic::Request<keymanagement::CountKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::CountKeyRingsResponse>, tonic::Status> {
        self.run_count_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several key rings at once.
    async fn resolve_key_rings(
        &self,
        request: tonic::Request<keymanagement::ResolveKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::ResolveKeyRingsResponse>, tonic::Status> {
        self.run_resolve_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportKeyRingsStream =
        tokio_stream::wrappers::ReceiverStream<Result<keymanagement::ExportKeyRingsResponse, tonic::Status>>;
    /// Streams every matching key ring.
    async fn export_key_rings(
        &self,
        request: tonic::Request<keymanagement::ExportKeyRingsRequest>,
    ) -> Result<tonic::Response<Self::ExportKeyRingsStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_export_key_rings(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Summarizes a key ring.
    async fn summarize_key_rings(
        &self,
        request: tonic::Request<keymanagement::SummarizeKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::SummarizeKeyRingsResponse>, tonic::Status> {
        self.run_summarize_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists key rings a page at a time.
    async fn suggest_key_rings(
        &self,
        request: tonic::Request<keymanagement::SuggestKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::SuggestKeyRingsResponse>, tonic::Status> {
        self.run_suggest_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key ring.
    async fn peek_key_ring(
        &self,
        request: tonic::Request<keymanagement::PeekKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::PeekKeyRingResponse>, tonic::Status> {
        self.run_peek_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
