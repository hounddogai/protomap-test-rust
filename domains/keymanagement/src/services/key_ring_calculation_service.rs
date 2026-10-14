use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_query_service_client::RoleBindingQueryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::csr_calculation_service_client::CsrCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_validation_service_client::ConfigChangeValidationServiceClient;
use acme_proto::acme::configuration::v1::config_set_query_service_client::ConfigSetQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_query_service_client::FeatureFlagQueryServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_validation_service_client::FeatureFlagValidationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_server::KeyRingCalculationService;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_query_service_client::SearchIndexQueryServiceClient;

/// Computes the figures of a key ring, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct KeyRingCalculationHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    compare_key_ring_options_2_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                configuration::ScoreConfigChangeRequest,
                configuration::ScoreConfigChangeResponse,
            >,
    >,
    compare_key_ring_options_3_port: acme_platform::ports::Callback<
        certificates::EstimateCertificateRequest,
        certificates::EstimateCertificateResponse,
    >,
    estimate_key_ring_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::EstimateQuotaBucketRequest,
            ratelimiting::EstimateQuotaBucketResponse,
        >,
    >,
    explain_key_ring_calculation_0_port: acme_platform::ports::Callback<
        authorization::BrowseRoleBindingsRequest,
        authorization::BrowseRoleBindingsResponse,
    >,
    recalculate_key_rings_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<configuration::CountConfigSetsRequest, configuration::CountConfigSetsResponse>,
    >,
    recalculate_key_rings_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::CalculateCryptoKeyRequest,
                keymanagement::CalculateCryptoKeyResponse,
            >,
    >,
    recalculate_key_rings_2_port: acme_platform::ports::Callback<
        featureflags::ExplainFeatureFlagDecisionRequest,
        featureflags::ExplainFeatureFlagDecisionResponse,
    >,
    simulate_key_ring_1_port: acme_platform::ports::Callback<
        featureflags::UnsubscribeFlagRolloutUpdatesRequest,
        featureflags::UnsubscribeFlagRolloutUpdatesResponse,
    >,
}
impl KeyRingCalculationHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::EstimateQuotaBucketRequest,
            ratelimiting::EstimateQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::EstimateQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::EstimateQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_quota_bucket(request).await?.into_inner())
})
})
        };
        let estimate_key_ring_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        let callback: acme_platform::ports::Callback<
            configuration::ScoreConfigChangeRequest,
            configuration::ScoreConfigChangeResponse,
        > = {
            let client = ConfigChangeValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::ScoreConfigChangeRequest| -> acme_platform::ports::RpcFuture<configuration::ScoreConfigChangeResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.score_config_change(request).await?.into_inner())
})
})
        };
        let compare_key_ring_options_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
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
        let compare_key_ring_options_3_port = callback;
        let callback: acme_platform::ports::Callback<
            configuration::CountConfigSetsRequest,
            configuration::CountConfigSetsResponse,
        > = {
            let client = ConfigSetQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: configuration::CountConfigSetsRequest| -> acme_platform::ports::RpcFuture<configuration::CountConfigSetsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.count_config_sets(request).await?.into_inner())
})
})
        };
        let recalculate_key_rings_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::CalculateCryptoKeyRequest,
            keymanagement::CalculateCryptoKeyResponse,
        > = {
            let client = CryptoKeyCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::CalculateCryptoKeyRequest| -> acme_platform::ports::RpcFuture<keymanagement::CalculateCryptoKeyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_crypto_key(request).await?.into_inner())
})
})
        };
        let recalculate_key_rings_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            featureflags::ExplainFeatureFlagDecisionRequest,
            featureflags::ExplainFeatureFlagDecisionResponse,
        > = {
            let client = FeatureFlagValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::ExplainFeatureFlagDecisionRequest| -> acme_platform::ports::RpcFuture<featureflags::ExplainFeatureFlagDecisionResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.explain_feature_flag_decision(request).await?.into_inner())
})
})
        };
        let recalculate_key_rings_2_port = callback;
        let callback: acme_platform::ports::Callback<
            featureflags::UnsubscribeFlagRolloutUpdatesRequest,
            featureflags::UnsubscribeFlagRolloutUpdatesResponse,
        > = {
            let client = FlagRolloutNotificationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::UnsubscribeFlagRolloutUpdatesRequest| -> acme_platform::ports::RpcFuture<featureflags::UnsubscribeFlagRolloutUpdatesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.unsubscribe_flag_rollout_updates(request).await?.into_inner())
})
})
        };
        let simulate_key_ring_1_port = callback;
        let callback: acme_platform::ports::Callback<
            authorization::BrowseRoleBindingsRequest,
            authorization::BrowseRoleBindingsResponse,
        > = {
            let client = RoleBindingQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: authorization::BrowseRoleBindingsRequest| -> acme_platform::ports::RpcFuture<authorization::BrowseRoleBindingsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_role_bindings(request).await?.into_inner())
})
})
        };
        let explain_key_ring_calculation_0_port = callback;
        Self {
            channel,
            repository,
            compare_key_ring_options_2_port,
            compare_key_ring_options_3_port,
            estimate_key_ring_0_port,
            explain_key_ring_calculation_0_port,
            recalculate_key_rings_0_port,
            recalculate_key_rings_1_port,
            recalculate_key_rings_2_port,
            simulate_key_ring_1_port,
        }
    }
    /// Run CalculateKeyRing after validating the tenant and durably recording the command.
    async fn run_calculate_key_ring(
        &self,
        req: keymanagement::CalculateKeyRingRequest,
    ) -> Result<keymanagement::CalculateKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ResolveFeatureFlagConflictRequest {
            context: req.context.clone(),
            details: Some(featureflags::resolve_feature_flag_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    created_at: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    created_by: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    updated_at: req
                        .key_ring
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FeatureFlagSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ResolveFeatureFlagConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_feature_flag_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::CalculateKeyRingResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run EstimateKeyRing after validating the tenant and durably recording the command.
    async fn run_estimate_key_ring(
        &self,
        req: keymanagement::EstimateKeyRingRequest,
    ) -> Result<keymanagement::EstimateKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::EstimateQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::estimate_quota_bucket_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    lines: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    recipient: req
                        .key_ring
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
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .key_ring
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
        let reply_0 =
            self.estimate_key_ring_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = keymanagement::EstimateKeyRingResponse::default();
        let contribution: Option<common::Money> = reply_0.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .display_name
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
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run QuoteKeyRing after validating the tenant and durably recording the command.
    async fn run_quote_key_ring(
        &self,
        req: keymanagement::QuoteKeyRingRequest,
    ) -> Result<keymanagement::QuoteKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ExportFeatureFlagsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::export_feature_flags_request::Filter {
                created_after: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.key_ring.clone().unwrap_or_default().attributes.clone(),
                owner_id: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .key_ring
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
        let _reply_0 = async move {
            let mut client = FeatureFlagQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::ExportFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_feature_flags(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::ExportFeatureFlagsResponse::default();
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
        let outgoing_1 = keymanagement::SimulateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    etag: req
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
                holder: Some(keymanagement::KeyRingRef {
                    display_name: req
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
            details: Some(keymanagement::simulate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    locality: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .locality
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SimulateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::CountFeatureFlagsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::count_feature_flags_request::Filter {
                created_after: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_1.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: reply_1.result.clone().unwrap_or_default().correlation_id.clone(),
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
        let reply_2 = async move {
            let mut client = FeatureFlagQueryServiceClient::new(channel);
            let input: tonic::Request<featureflags::CountFeatureFlagsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_feature_flags(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::QuoteKeyRingResponse::default();
        let contribution: Option<prost_types::Timestamp> = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
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
            if response.key_ring.get_or_insert_with(Default::default).children[0]
                .children
                .is_empty()
            {
                response.key_ring.get_or_insert_with(Default::default).children[0]
                    .children
                    .push(Default::default());
            }
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0]
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run CompareKeyRingOptions after validating the tenant and durably recording the command.
    async fn run_compare_key_ring_options(
        &self,
        req: keymanagement::CompareKeyRingOptionsRequest,
    ) -> Result<keymanagement::CompareKeyRingOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CompareKeyRingOptions", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareKeyRingOptions", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::RecalculateCryptoKeysRequest {
            context: req.context.clone(),
            details: Some(keymanagement::recalculate_crypto_keys_request::Details {
                destination: Some(common::PostalAddress {
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
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .locality
                        .clone(),
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    region: req
                        .key_ring
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
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::RecalculateCryptoKeysRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_crypto_keys(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = search::ExportSearchIndexesRequest {
            context: req.context.clone(),
            filter: Some(search::export_search_indexes_request::Filter {
                labels: reply_0.crypto_key.clone().unwrap_or_default().labels.clone(),
                region: req
                    .key_ring
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
        let _reply_1 = async move {
            let mut client = SearchIndexQueryServiceClient::new(channel);
            let input: tonic::Request<search::ExportSearchIndexesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_search_indexes(input).await?;
            let mut frames = response.into_inner();
            let mut last = search::ExportSearchIndexesResponse::default();
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
        let outgoing_2 = configuration::ScoreConfigChangeRequest {
            context: req.context.clone(),
            details: Some(configuration::score_config_change_request::Details {
                destination: Some(common::PostalAddress {
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
            }),
            ..Default::default()
        };
        let reply_2 = acme_platform::ports::execute(self.compare_key_ring_options_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = certificates::EstimateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    etag: req
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
                counterparty: Some(certificates::CsrRef {
                    display_name: reply_2
                        .config_change
                        .clone()
                        .unwrap_or_default()
                        .subject
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
        let reply_3 = (self.compare_key_ring_options_3_port)(outgoing_3).await?;
        let mut response = keymanagement::CompareKeyRingOptionsResponse::default();
        let contribution: String = reply_0.crypto_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_2
            .config_change
            .clone()
            .unwrap_or_default()
            .subject
            .clone()
            .unwrap_or_default()
            .display_name
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
                .display_name = contribution;
        }
        let contribution: String = reply_2.config_change.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Option<common::Money> = reply_3.certificate.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: String = reply_3
            .certificate
            .clone()
            .unwrap_or_default()
            .billing_address
            .clone()
            .unwrap_or_default()
            .region
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .location
                .get_or_insert_with(Default::default)
                .region = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).parent_id = contribution;
        }
        Ok(response)
    }
    /// Run RecalculateKeyRings after validating the tenant and durably recording the command.
    async fn run_recalculate_key_rings(
        &self,
        req: keymanagement::RecalculateKeyRingsRequest,
    ) -> Result<keymanagement::RecalculateKeyRingsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecalculateKeyRings", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateKeyRings", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CountConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_sets_request::Filter {
                created_after: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                region: req
                    .key_ring
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
        let reply_0 = acme_platform::ports::execute(self.recalculate_key_rings_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = keymanagement::CalculateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
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
            details: Some(keymanagement::calculate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .postal_code
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
            }),
            ..Default::default()
        };
        let reply_1 = acme_platform::ports::execute(self.recalculate_key_rings_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = featureflags::ExplainFeatureFlagDecisionRequest {
            context: req.context.clone(),
            details: Some(featureflags::explain_feature_flag_decision_request::Details {
                contact: Some(common::ContactInfo {
                    phone: reply_1
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .phone
                        .clone(),
                    preferred_channel: reply_1
                        .crypto_key
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
            feature_flag: Some(featureflags::FeatureFlag {
                audit: Some(common::AuditInfo {
                    etag: reply_1
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .etag
                        .clone(),
                    ..Default::default()
                }),
                owner: Some(identity::UserRef {
                    email: req
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
        let reply_2 = (self.recalculate_key_rings_2_port)(outgoing_2).await?;
        let mut response = keymanagement::RecalculateKeyRingsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .manager
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        let contribution: String = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .display_name
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
                .manager
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2
            .feature_flag
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.key_ring.clone().unwrap_or_default().parent_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).parent_id = contribution;
        }
        Ok(response)
    }
    /// Run SimulateKeyRing after validating the tenant and durably recording the command.
    async fn run_simulate_key_ring(
        &self,
        req: keymanagement::SimulateKeyRingRequest,
    ) -> Result<keymanagement::SimulateKeyRingResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateKeyRing", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateKeyRing", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::EstimateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .key_ring
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
            details: Some(keymanagement::estimate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    lines: (req
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone())
                    .into_iter()
                    .filter_map(|value| {
                        let value = value.trim();
                        if value.is_empty() { None } else { Some(value.to_owned()) }
                    })
                    .collect(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::EstimateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::UnsubscribeFlagRolloutUpdatesRequest {
            context: req.context.clone(),
            details: Some(featureflags::unsubscribe_flag_rollout_updates_request::Details {
                recipient: Some(common::ContactInfo {
                    name: reply_0
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .contact
                        .clone()
                        .unwrap_or_default()
                        .name
                        .clone(),
                    preferred_channel: reply_0
                        .crypto_key
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
        let port = acme_platform::ports::opaque(self.simulate_key_ring_1_port.clone());
        let reply_1 = acme_platform::ports::execute(&port, outgoing_1).await?;
        let outgoing_2 = certificates::ExportCertificateAuthoritiesRequest {
            context: req.context.clone(),
            filter: Some(certificates::export_certificate_authorities_request::Filter {
                created_after: reply_1
                    .flag_rollout
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: reply_0.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
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
            let mut client = CertificateAuthorityQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ExportCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.export_certificate_authorities(input).await?;
            let mut frames = response.into_inner();
            let mut last = certificates::ExportCertificateAuthoritiesResponse::default();
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
        let mut command_3 = SimulateKeyRingDispatch3::from(req.clone());
        command_3.details_destination_postal_code = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .postal_code
            .clone();
        let outgoing_3: certificates::CalculateCsrRequest = command_3.into();
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = CsrCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CalculateCsrRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::SimulateKeyRingResponse::default();
        let contribution: String = reply_0
            .crypto_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .key_ring_id
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
            response.key_ring.get_or_insert_with(Default::default).children[0].children[0].key_ring_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .flag_rollout
            .clone()
            .unwrap_or_default()
            .location
            .clone()
            .unwrap_or_default()
            .region
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
                .region = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ExplainKeyRingCalculation after validating the tenant and durably recording the command.
    async fn run_explain_key_ring_calculation(
        &self,
        req: keymanagement::ExplainKeyRingCalculationRequest,
    ) -> Result<keymanagement::ExplainKeyRingCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainKeyRingCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainKeyRingCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.key_ring.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = authorization::BrowseRoleBindingsRequest {
            context: req.context.clone(),
            filter: Some(authorization::browse_role_bindings_request::Filter {
                created_after: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.annotations.clone(),
                owner_id: req
                    .key_ring
                    .clone()
                    .unwrap_or_default()
                    .manager
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: req
                    .key_ring
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
        let port = acme_platform::ports::opaque(self.explain_key_ring_calculation_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = keymanagement::ExplainCryptoKeyCalculationRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
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
        let outgoing_1 =
            acme_platform::ports::RequestBuilder::<keymanagement::ExplainCryptoKeyCalculationRequest>::default()
                .map(|draft| *draft = outgoing_1)
                .deadline(std::time::Duration::from_secs(8))
                .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::ExplainCryptoKeyCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_crypto_key_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut command_2 = ExplainKeyRingCalculationDispatch2::from(req.clone());
        command_2.crypto_key_audit_updated_by = reply_1
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        let outgoing_2: keymanagement::SimulateCryptoKeyRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SimulateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ExplainKeyRingCalculationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: Vec<String> = reply_1.result.clone().unwrap_or_default().reference_ids.clone();
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
                .etag = contribution;
        }
        let contribution: String = reply_2
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
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
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_2
            .crypto_key
            .clone()
            .unwrap_or_default()
            .contact
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
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        let contribution: Option<common::PostalAddress> = req.key_ring.clone().unwrap_or_default().location.clone();
        if contribution != <Option<common::PostalAddress>>::default() {
            response.key_ring.get_or_insert_with(Default::default).location = contribution;
        }
        Ok(response)
    }
    /// Run GetKeyRingRates after validating the tenant and durably recording the command.
    async fn run_get_key_ring_rates(
        &self,
        req: keymanagement::GetKeyRingRatesRequest,
    ) -> Result<keymanagement::GetKeyRingRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetKeyRingRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetKeyRingRates", "running workflow");
        let outgoing_0 = configuration::ValidateConfigChangeRequest {
            config_change: Some(configuration::ConfigChange {
                correlation_id: req.key_ring_id.clone(),
                ..Default::default()
            }),
            context: req.context.clone(),
            details: Some(configuration::validate_config_change_request::Details {
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
        let outgoing_0 =
            acme_platform::codec::through_any(&outgoing_0, "acme.configuration.v1.ValidateConfigChangeRequest")?;
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigChangeValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateConfigChangeRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_config_change(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::CountConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_sets_request::Filter {
                created_after: reply_0.outcome.clone().unwrap_or_default().effective_at.clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0
                    .config_change
                    .clone()
                    .unwrap_or_default()
                    .subject
                    .clone()
                    .unwrap_or_default()
                    .config_set_id
                    .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<configuration::CountConfigSetsRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = ConfigSetQueryServiceClient::new(channel);
                let input: tonic::Request<configuration::CountConfigSetsRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_config_sets(input).await?;
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
        let mut response = keymanagement::GetKeyRingRatesResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.config_change.clone().unwrap_or_default().attributes.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .key_ring
                .get_or_insert_with(Default::default)
                .attributes
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.key_ring_id.clone();
        if contribution != <String>::default() {
            response.key_ring.get_or_insert_with(Default::default).key_ring_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by SimulateKeyRing for CalculateCsr.
struct SimulateKeyRingDispatch3 {
    context: Option<common::RequestContext>,
    details_destination_postal_code: String,
}
impl From<keymanagement::SimulateKeyRingRequest> for SimulateKeyRingDispatch3 {
    fn from(req: keymanagement::SimulateKeyRingRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_destination_postal_code: Default::default(),
        }
    }
}
impl From<SimulateKeyRingDispatch3> for certificates::CalculateCsrRequest {
    fn from(value: SimulateKeyRingDispatch3) -> Self {
        certificates::CalculateCsrRequest {
            context: value.context,
            details: Some(certificates::calculate_csr_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: value.details_destination_postal_code,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
/// The domain command prepared by ExplainKeyRingCalculation for SimulateCryptoKey.
struct ExplainKeyRingCalculationDispatch2 {
    context: Option<common::RequestContext>,
    details_destination_locality: String,
    details_destination_country_code: String,
    crypto_key_audit_updated_by: String,
}
impl From<keymanagement::ExplainKeyRingCalculationRequest> for ExplainKeyRingCalculationDispatch2 {
    fn from(req: keymanagement::ExplainKeyRingCalculationRequest) -> Self {
        Self {
            context: req.context.clone(),
            details_destination_locality: req
                .key_ring
                .clone()
                .unwrap_or_default()
                .location
                .clone()
                .unwrap_or_default()
                .locality
                .clone(),
            details_destination_country_code: req
                .key_ring
                .clone()
                .unwrap_or_default()
                .location
                .clone()
                .unwrap_or_default()
                .country_code
                .clone(),
            crypto_key_audit_updated_by: Default::default(),
        }
    }
}
impl From<ExplainKeyRingCalculationDispatch2> for keymanagement::SimulateCryptoKeyRequest {
    fn from(value: ExplainKeyRingCalculationDispatch2) -> Self {
        keymanagement::SimulateCryptoKeyRequest {
            context: value.context,
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    updated_by: value.crypto_key_audit_updated_by,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            details: Some(keymanagement::simulate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    country_code: value.details_destination_country_code,
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
impl KeyRingCalculationService for KeyRingCalculationHandler {
    /// Calculates a key ring.
    async fn calculate_key_ring(
        &self,
        request: tonic::Request<keymanagement::CalculateKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::CalculateKeyRingResponse>, tonic::Status> {
        self.run_calculate_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a key ring.
    async fn estimate_key_ring(
        &self,
        request: tonic::Request<keymanagement::EstimateKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::EstimateKeyRingResponse>, tonic::Status> {
        self.run_estimate_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a key ring.
    async fn quote_key_ring(
        &self,
        request: tonic::Request<keymanagement::QuoteKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::QuoteKeyRingResponse>, tonic::Status> {
        self.run_quote_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a key ring.
    async fn compare_key_ring_options(
        &self,
        request: tonic::Request<keymanagement::CompareKeyRingOptionsRequest>,
    ) -> Result<tonic::Response<keymanagement::CompareKeyRingOptionsResponse>, tonic::Status> {
        self.run_compare_key_ring_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a key ring.
    async fn recalculate_key_rings(
        &self,
        request: tonic::Request<keymanagement::RecalculateKeyRingsRequest>,
    ) -> Result<tonic::Response<keymanagement::RecalculateKeyRingsResponse>, tonic::Status> {
        self.run_recalculate_key_rings(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a key ring.
    async fn simulate_key_ring(
        &self,
        request: tonic::Request<keymanagement::SimulateKeyRingRequest>,
    ) -> Result<tonic::Response<keymanagement::SimulateKeyRingResponse>, tonic::Status> {
        self.run_simulate_key_ring(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a key ring.
    async fn explain_key_ring_calculation(
        &self,
        request: tonic::Request<keymanagement::ExplainKeyRingCalculationRequest>,
    ) -> Result<tonic::Response<keymanagement::ExplainKeyRingCalculationResponse>, tonic::Status> {
        self.run_explain_key_ring_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key ring.
    async fn get_key_ring_rates(
        &self,
        request: tonic::Request<keymanagement::GetKeyRingRatesRequest>,
    ) -> Result<tonic::Response<keymanagement::GetKeyRingRatesResponse>, tonic::Status> {
        self.run_get_key_ring_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
