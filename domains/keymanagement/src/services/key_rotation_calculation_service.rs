use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_rotation_calculation_service_server::KeyRotationCalculationService;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_query_service_client::MetricSeriesQueryServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_client::QuotaBucketQueryServiceClient;
use acme_proto::acme::search::v1 as search;
use acme_proto::acme::search::v1::search_index_service_client::SearchIndexServiceClient;

/// Computes the figures of a key rotation, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct KeyRotationCalculationEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    calculate_key_rotation_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::EstimateCryptoKeyRequest,
                keymanagement::EstimateCryptoKeyResponse,
            >,
    >,
    calculate_key_rotation_1_port: acme_platform::ports::Callback<
        keymanagement::SimulateCryptoKeyRequest,
        keymanagement::SimulateCryptoKeyResponse,
    >,
    recalculate_key_rotations_2_port: std::sync::Arc<
        acme_platform::ports::Adapter<audit::BrowseAuditEventsRequest, audit::BrowseAuditEventsResponse>,
    >,
    recalculate_key_rotations_3_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                keymanagement::EstimateCryptoKeyRequest,
                keymanagement::EstimateCryptoKeyResponse,
            >,
    >,
    simulate_key_rotation_0_port: acme_platform::ports::Callback<
        keymanagement::RecalculateCryptoKeysRequest,
        keymanagement::RecalculateCryptoKeysResponse,
    >,
    simulate_key_rotation_1_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<search::BatchGetSearchIndexesRequest, search::BatchGetSearchIndexesResponse>,
    >,
}
impl KeyRotationCalculationEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::EstimateCryptoKeyRequest,
            keymanagement::EstimateCryptoKeyResponse,
        > = {
            let client = CryptoKeyCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::EstimateCryptoKeyRequest| -> acme_platform::ports::RpcFuture<keymanagement::EstimateCryptoKeyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_crypto_key(request).await?.into_inner())
})
})
        };
        let calculate_key_rotation_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::SimulateCryptoKeyRequest,
            keymanagement::SimulateCryptoKeyResponse,
        > = {
            let client = CryptoKeyCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::SimulateCryptoKeyRequest| -> acme_platform::ports::RpcFuture<keymanagement::SimulateCryptoKeyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.simulate_crypto_key(request).await?.into_inner())
})
})
        };
        let calculate_key_rotation_1_port = callback;
        let callback: acme_platform::ports::Callback<
            audit::BrowseAuditEventsRequest,
            audit::BrowseAuditEventsResponse,
        > = {
            let client = AuditEventQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: audit::BrowseAuditEventsRequest| -> acme_platform::ports::RpcFuture<audit::BrowseAuditEventsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.browse_audit_events(request).await?.into_inner())
})
})
        };
        let recalculate_key_rotations_2_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::EstimateCryptoKeyRequest,
            keymanagement::EstimateCryptoKeyResponse,
        > = {
            let client = CryptoKeyCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::EstimateCryptoKeyRequest| -> acme_platform::ports::RpcFuture<keymanagement::EstimateCryptoKeyResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_crypto_key(request).await?.into_inner())
})
})
        };
        let recalculate_key_rotations_3_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::RecalculateCryptoKeysRequest,
            keymanagement::RecalculateCryptoKeysResponse,
        > = {
            let client = CryptoKeyCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::RecalculateCryptoKeysRequest| -> acme_platform::ports::RpcFuture<keymanagement::RecalculateCryptoKeysResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.recalculate_crypto_keys(request).await?.into_inner())
})
})
        };
        let simulate_key_rotation_0_port = callback;
        let callback: acme_platform::ports::Callback<
            search::BatchGetSearchIndexesRequest,
            search::BatchGetSearchIndexesResponse,
        > = {
            let client = SearchIndexServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: search::BatchGetSearchIndexesRequest| -> acme_platform::ports::RpcFuture<search::BatchGetSearchIndexesResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.batch_get_search_indexes(request).await?.into_inner())
})
})
        };
        let simulate_key_rotation_1_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            calculate_key_rotation_0_port,
            calculate_key_rotation_1_port,
            recalculate_key_rotations_2_port,
            recalculate_key_rotations_3_port,
            simulate_key_rotation_0_port,
            simulate_key_rotation_1_port,
        }
    }
    /// Run CalculateKeyRotation after validating the tenant and durably recording the command.
    async fn run_calculate_key_rotation(
        &self,
        req: keymanagement::CalculateKeyRotationRequest,
    ) -> Result<keymanagement::CalculateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::EstimateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
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
                ..Default::default()
            }),
            details: Some(keymanagement::estimate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    recipient: req
                        .key_rotation
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
        let reply_0 = acme_platform::ports::execute(self.calculate_key_rotation_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = keymanagement::SimulateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
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
                funding: Some(keymanagement::crypto_key::Funding::Card(
                    keymanagement::CryptoKeyCardFunding {
                        network: (match reply_0.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        ..Default::default()
                    },
                )),
                contact: Some(common::ContactInfo {
                    email: reply_0
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
                holder: Some(keymanagement::KeyRingRef {
                    path: reply_0
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .holder
                        .clone()
                        .unwrap_or_default()
                        .path
                        .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_1 = (self.calculate_key_rotation_1_port)(outgoing_1).await?;
        let mut response = keymanagement::CalculateKeyRotationResponse::default();
        let contribution: String = reply_0
            .crypto_key
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
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
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
    /// Run EstimateKeyRotation after validating the tenant and durably recording the command.
    async fn run_estimate_key_rotation(
        &self,
        req: keymanagement::EstimateKeyRotationRequest,
    ) -> Result<keymanagement::EstimateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let jobs_0 = req
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
                    let outgoing_0 = keymanagement::SimulateCryptoKeyRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            contact: Some(common::ContactInfo {
                                email: item.contact.clone().unwrap_or_default().email.clone(),
                                name: item.contact.clone().unwrap_or_default().name.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyCalculationServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::SimulateCryptoKeyRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.simulate_crypto_key(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::EstimateKeyRotationResponse::default();
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
    /// Run RecalculateKeyRotations after validating the tenant and durably recording the command.
    async fn run_recalculate_key_rotations(
        &self,
        req: keymanagement::RecalculateKeyRotationsRequest,
    ) -> Result<keymanagement::RecalculateKeyRotationsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("RecalculateKeyRotations", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateKeyRotations", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::RecalculateKeyRingsRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        location: Some(common::PostalAddress {
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
                    }],
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
        let outgoing_1 = ratelimiting::RecalculateQuotaBucketsRequest {
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::RecalculateQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.recalculate_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = audit::BrowseAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::browse_audit_events_request::Filter {
                labels: reply_1.quota_bucket.clone().unwrap_or_default().labels.clone(),
                region: reply_0
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
        let reply_2 = acme_platform::ports::execute(self.recalculate_key_rotations_2_port.as_ref(), outgoing_2).await?;
        let outgoing_3 = keymanagement::EstimateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
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
                    updated_by: reply_1
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_3 = acme_platform::ports::execute(self.recalculate_key_rotations_3_port.as_ref(), outgoing_3).await?;
        let mut response = keymanagement::RecalculateKeyRotationsResponse::default();
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
        let contribution: Option<prost_types::Timestamp> = reply_1
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
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
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
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<common::Money> = reply_3.breakdown.clone().unwrap_or_default().total.clone();
        if contribution != <Option<common::Money>>::default() {
            response.breakdown.get_or_insert_with(Default::default).total = contribution;
        }
        let contribution: Vec<String> = reply_3.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
    /// Run SimulateKeyRotation after validating the tenant and durably recording the command.
    async fn run_simulate_key_rotation(
        &self,
        req: keymanagement::SimulateKeyRotationRequest,
    ) -> Result<keymanagement::SimulateKeyRotationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateKeyRotation", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateKeyRotation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::RecalculateCryptoKeysRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
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
                ..Default::default()
            }),
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
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    recipient: req
                        .key_rotation
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
        let reply_0 = (self.simulate_key_rotation_0_port)(outgoing_0).await?;
        let outgoing_1 = search::BatchGetSearchIndexesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let reply_1 =
            self.simulate_key_rotation_1_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_1)
            .await?;
        let mut response = keymanagement::SimulateKeyRotationResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
            response.result.get_or_insert_with(Default::default).message = contribution;
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
    /// Run ExplainKeyRotationCalculation after validating the tenant and durably recording the command.
    async fn run_explain_key_rotation_calculation(
        &self,
        req: keymanagement::ExplainKeyRotationCalculationRequest,
    ) -> Result<keymanagement::ExplainKeyRotationCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainKeyRotationCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainKeyRotationCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.key_rotation.clone().unwrap_or_default().key_rotation_id.clone(), "recorded audit field");
        let outgoing_0 = search::BatchCreateSearchIndexesRequest {
            context: req.context.clone(),
            chunk: Some(search::batch_create_search_indexes_request::Chunk::Record(
                search::SearchIndex {
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let _reply_0 = async move {
            let mut client = SearchIndexServiceClient::new(channel);
            let input: tonic::Request<search::BatchCreateSearchIndexesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
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
        let outgoing_1 = metrics::CountMetricSeriesesRequest {
            context: req.context.clone(),
            filter: Some(metrics::count_metric_serieses_request::Filter {
                created_after: req
                    .key_rotation
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: (req.annotations.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                owner_id: req.key_rotation_id.clone(),
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
            let mut client = MetricSeriesQueryServiceClient::new(channel);
            let input: tonic::Request<metrics::CountMetricSeriesesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.count_metric_serieses(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_2 = ExplainKeyRotationCalculationDispatch2::from(req.clone());
        let outgoing_2: ratelimiting::SummarizeQuotaBucketsRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = QuotaBucketQueryServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::SummarizeQuotaBucketsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_quota_buckets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = keymanagement::SimulateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                contact: Some(common::ContactInfo {
                    email: req
                        .key_rotation
                        .clone()
                        .unwrap_or_default()
                        .organizer
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
        let outgoing_3 =
            acme_platform::codec::through_any(&outgoing_3, "acme.keymanagement.v1.SimulateCryptoKeyRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SimulateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ExplainKeyRotationCalculationResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_3
            .crypto_key
            .clone()
            .unwrap_or_default()
            .holder
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .organizer
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
            .crypto_key
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .key_rotation
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
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
    /// Run GetKeyRotationRates after validating the tenant and durably recording the command.
    async fn run_get_key_rotation_rates(
        &self,
        req: keymanagement::GetKeyRotationRatesRequest,
    ) -> Result<keymanagement::GetKeyRotationRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetKeyRotationRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetKeyRotationRates", "running workflow");
        let outgoing_0 = keymanagement::SimulateCryptoKeyRequest {
            context: req.context.clone(),
            crypto_key: Some(keymanagement::CryptoKey {
                audit: Some(common::AuditInfo {
                    updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
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
                funding: Some(keymanagement::crypto_key::Funding::ExternalAccountId(
                    req.key_rotation_id.clone(),
                )),
                labels: req.context.clone().unwrap_or_default().baggage.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<keymanagement::SimulateCryptoKeyRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = CryptoKeyCalculationServiceClient::new(channel);
                let input: tonic::Request<keymanagement::SimulateCryptoKeyRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.simulate_crypto_key(input).await?;
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
        let outgoing_1 = search::BatchGetSearchIndexesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = SearchIndexServiceClient::new(channel);
            let input: tonic::Request<search::BatchGetSearchIndexesRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_search_indexes(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = keymanagement::GetKeyRotationRatesResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
        Ok(response)
    }
}
/// The domain command prepared by ExplainKeyRotationCalculation for SummarizeQuotaBuckets.
struct ExplainKeyRotationCalculationDispatch2 {
    context: Option<common::RequestContext>,
    reason: String,
}
impl From<keymanagement::ExplainKeyRotationCalculationRequest> for ExplainKeyRotationCalculationDispatch2 {
    fn from(req: keymanagement::ExplainKeyRotationCalculationRequest) -> Self {
        Self {
            context: req.context.clone(),
            reason: req.reason.clone(),
        }
    }
}
impl From<ExplainKeyRotationCalculationDispatch2> for ratelimiting::SummarizeQuotaBucketsRequest {
    fn from(value: ExplainKeyRotationCalculationDispatch2) -> Self {
        ratelimiting::SummarizeQuotaBucketsRequest {
            context: value.context,
            reason: value.reason,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl KeyRotationCalculationService for KeyRotationCalculationEndpoint {
    /// Calculates a key rotation.
    async fn calculate_key_rotation(
        &self,
        request: tonic::Request<keymanagement::CalculateKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::CalculateKeyRotationResponse>, tonic::Status> {
        self.run_calculate_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a key rotation.
    async fn estimate_key_rotation(
        &self,
        request: tonic::Request<keymanagement::EstimateKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::EstimateKeyRotationResponse>, tonic::Status> {
        self.run_estimate_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a key rotation.
    async fn recalculate_key_rotations(
        &self,
        request: tonic::Request<keymanagement::RecalculateKeyRotationsRequest>,
    ) -> Result<tonic::Response<keymanagement::RecalculateKeyRotationsResponse>, tonic::Status> {
        self.run_recalculate_key_rotations(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a key rotation.
    async fn simulate_key_rotation(
        &self,
        request: tonic::Request<keymanagement::SimulateKeyRotationRequest>,
    ) -> Result<tonic::Response<keymanagement::SimulateKeyRotationResponse>, tonic::Status> {
        self.run_simulate_key_rotation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a key rotation.
    async fn explain_key_rotation_calculation(
        &self,
        request: tonic::Request<keymanagement::ExplainKeyRotationCalculationRequest>,
    ) -> Result<tonic::Response<keymanagement::ExplainKeyRotationCalculationResponse>, tonic::Status> {
        self.run_explain_key_rotation_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a key rotation.
    async fn get_key_rotation_rates(
        &self,
        request: tonic::Request<keymanagement::GetKeyRotationRatesRequest>,
    ) -> Result<tonic::Response<keymanagement::GetKeyRotationRatesResponse>, tonic::Status> {
        self.run_get_key_rotation_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
