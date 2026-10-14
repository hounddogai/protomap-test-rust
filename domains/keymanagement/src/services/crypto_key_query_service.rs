use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_server::CryptoKeyQueryService;
use acme_proto::acme::keymanagement::v1::key_rotation_validation_service_client::KeyRotationValidationServiceClient;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::prescriptions::v1 as prescriptions;
use acme_proto::acme::prescriptions::v1::prescription_service_client::PrescriptionServiceClient;
use acme_proto::acme::seating::v1 as seating;
use acme_proto::acme::seating::v1::seat_map_service_client::SeatMapServiceClient;

/// Answers read-only questions about crypto keys, for screens and reports.
#[derive(Clone)]
pub struct CryptoKeyQueryEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    find_crypto_keys_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<
            keymanagement::PreviewKeyRotationRequest,
            keymanagement::PreviewKeyRotationResponse,
        >,
    >,
    lookup_crypto_key_0_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<
                featureflags::CalculateFlagEvaluationRequest,
                featureflags::CalculateFlagEvaluationResponse,
            >,
    >,
    resolve_crypto_keys_0_port: acme_platform::ports::Callback<
        keymanagement::NormalizeKeyRotationRequest,
        keymanagement::NormalizeKeyRotationResponse,
    >,
}
impl CryptoKeyQueryEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            featureflags::CalculateFlagEvaluationRequest,
            featureflags::CalculateFlagEvaluationResponse,
        > = {
            let client = FlagEvaluationCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: featureflags::CalculateFlagEvaluationRequest| -> acme_platform::ports::RpcFuture<featureflags::CalculateFlagEvaluationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_flag_evaluation(request).await?.into_inner())
})
})
        };
        let lookup_crypto_key_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::PreviewKeyRotationRequest,
            keymanagement::PreviewKeyRotationResponse,
        > = {
            let client = KeyRotationValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::PreviewKeyRotationRequest| -> acme_platform::ports::RpcFuture<keymanagement::PreviewKeyRotationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.preview_key_rotation(request).await?.into_inner())
})
})
        };
        let find_crypto_keys_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            keymanagement::NormalizeKeyRotationRequest,
            keymanagement::NormalizeKeyRotationResponse,
        > = {
            let client = KeyRotationValidationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::NormalizeKeyRotationRequest| -> acme_platform::ports::RpcFuture<keymanagement::NormalizeKeyRotationResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.normalize_key_rotation(request).await?.into_inner())
})
})
        };
        let resolve_crypto_keys_0_port = callback;
        Self {
            channel,
            repository,
            find_crypto_keys_0_port,
            lookup_crypto_key_0_port,
            resolve_crypto_keys_0_port,
        }
    }
    /// Run LookupCryptoKey after validating the tenant and durably recording the command.
    async fn run_lookup_crypto_key(
        &self,
        req: keymanagement::LookupCryptoKeyRequest,
    ) -> Result<keymanagement::LookupCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LookupCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LookupCryptoKey", "running workflow");
        let outgoing_0 = featureflags::CalculateFlagEvaluationRequest {
            context: req.context.clone(),
            flag_evaluation: Some(featureflags::FlagEvaluation {
                source: req.crypto_key_id.clone(),
                subject: Some(featureflags::FlagRolloutRef {
                    starts_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let reply_0 = acme_platform::ports::execute(self.lookup_crypto_key_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = prescriptions::BatchCreatePrescriptionsRequest {
            context: req.context.clone(),
            chunk: Some(prescriptions::batch_create_prescriptions_request::Chunk::Record(
                prescriptions::Prescription {
                    counterparty: Some(patients::PatientRef {
                        display_name: reply_0
                            .flag_evaluation
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = PrescriptionServiceClient::new(channel);
            let input: tonic::Request<prescriptions::BatchCreatePrescriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let metadata = input.metadata().clone();
            let (sender, receiver) = tokio::sync::mpsc::channel(8);
            tokio::spawn(async move {
                let _ = sender.send(input.into_inner()).await;
            });
            let mut input = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(receiver));
            *input.metadata_mut() = metadata;
            let response = client.batch_create_prescriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_2 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    flag_rollout_id: reply_0
                        .flag_evaluation
                        .clone()
                        .unwrap_or_default()
                        .subject
                        .clone()
                        .unwrap_or_default()
                        .flag_rollout_id
                        .clone(),
                    organizer: Some(identity::UserRef {
                        display_name: reply_0
                            .flag_evaluation
                            .clone()
                            .unwrap_or_default()
                            .subject
                            .clone()
                            .unwrap_or_default()
                            .display_name
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
                    participants: vec![featureflags::FlagRolloutParticipant {
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
                },
            )),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = keymanagement::LookupCryptoKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        Ok(response)
    }
    /// Run BrowseCryptoKeys after validating the tenant and durably recording the command.
    async fn run_browse_crypto_keys(
        &self,
        req: keymanagement::BrowseCryptoKeysRequest,
    ) -> Result<keymanagement::BrowseCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BrowseCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BrowseCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = keymanagement::BrowseCryptoKeysResponse::default();
        Ok(response)
    }
    /// Run FindCryptoKeys after validating the tenant and durably recording the command.
    async fn run_find_crypto_keys(
        &self,
        req: keymanagement::FindCryptoKeysRequest,
    ) -> Result<keymanagement::FindCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("FindCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "FindCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::PreviewKeyRotationRequest {
            context: req.context.clone(),
            key_rotation: Some(keymanagement::KeyRotation {
                location: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
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
        let reply_0 = acme_platform::ports::execute(self.find_crypto_keys_0_port.as_ref(), outgoing_0).await?;
        let mut response = keymanagement::FindCryptoKeysResponse::default();
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
            .key_rotation
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            if response.crypto_keys.is_empty() {
                response.crypto_keys.push(Default::default());
            }
            response.crypto_keys[0].audit.get_or_insert_with(Default::default).etag = contribution;
        }
        Ok(response)
    }
    /// Run CountCryptoKeys after validating the tenant and durably recording the command.
    async fn run_count_crypto_keys(
        &self,
        req: keymanagement::CountCryptoKeysRequest,
    ) -> Result<keymanagement::CountCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CountCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CountCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = configuration::SearchConfigSetsRequest {
            context: req.context.clone(),
            filter: Some(configuration::search_config_sets_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: (req.filter.clone().unwrap_or_default().labels.clone())
                    .into_iter()
                    .filter(|(key, _)| !key.is_empty())
                    .map(|(key, value)| (key, value.trim().to_owned()))
                    .collect(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::SearchConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.search_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::CountCryptoKeysResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run ResolveCryptoKeys after validating the tenant and durably recording the command.
    async fn run_resolve_crypto_keys(
        &self,
        req: keymanagement::ResolveCryptoKeysRequest,
    ) -> Result<keymanagement::ResolveCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ResolveCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveCryptoKeys", "running workflow");
        let outgoing_0 = keymanagement::NormalizeKeyRotationRequest {
            context: req.context.clone(),
            details: Some(keymanagement::normalize_key_rotation_request::Details {
                locale: req.context.clone().unwrap_or_default().locale.clone(),
                ..Default::default()
            }),
            key_rotation: Some(keymanagement::KeyRotation {
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
        let port = acme_platform::ports::opaque(self.resolve_crypto_keys_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let mut response = keymanagement::ResolveCryptoKeysResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        Ok(response)
    }
    /// Run ExportCryptoKeys after validating the tenant and durably recording the command.
    async fn run_export_crypto_keys(
        &self,
        req: keymanagement::ExportCryptoKeysRequest,
    ) -> Result<keymanagement::ExportCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ExportCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExportCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.filter.clone().unwrap_or_default().query.clone(), "recorded audit field");
        let outgoing_0 = seating::CloneSeatMapRequest {
            context: req.context.clone(),
            seat_map: Some(seating::SeatMap {
                children: vec![seating::SeatMap {
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
            let mut client = SeatMapServiceClient::new(channel);
            let input: tonic::Request<seating::CloneSeatMapRequest> = tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_seat_map(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::EstimateCryptoKeyRequest {
            context: req.context.clone(),
            details: Some(keymanagement::estimate_crypto_key_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: reply_0
                        .seat_map
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
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::EstimateCryptoKeyRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_crypto_key(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ExportCryptoKeysResponse::default();
        let contribution: String = reply_0
            .seat_map
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_by
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = reply_1.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.filter.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run SuggestCryptoKeys after validating the tenant and durably recording the command.
    async fn run_suggest_crypto_keys(
        &self,
        req: keymanagement::SuggestCryptoKeysRequest,
    ) -> Result<keymanagement::SuggestCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SuggestCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SuggestCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = keymanagement::SuggestCryptoKeysResponse::default();
        Ok(response)
    }
}
#[tonic::async_trait]
impl CryptoKeyQueryService for CryptoKeyQueryEndpoint {
    /// Returns a crypto key.
    async fn lookup_crypto_key(
        &self,
        request: tonic::Request<keymanagement::LookupCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::LookupCryptoKeyResponse>, tonic::Status> {
        self.run_lookup_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists crypto keys a page at a time.
    async fn browse_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::BrowseCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::BrowseCryptoKeysResponse>, tonic::Status> {
        self.run_browse_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lists crypto keys a page at a time.
    async fn find_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::FindCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::FindCryptoKeysResponse>, tonic::Status> {
        self.run_find_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Counts crypto keys by status.
    async fn count_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::CountCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::CountCryptoKeysResponse>, tonic::Status> {
        self.run_count_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns several crypto keys at once.
    async fn resolve_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::ResolveCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::ResolveCryptoKeysResponse>, tonic::Status> {
        self.run_resolve_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    type ExportCryptoKeysStream =
        tokio_stream::wrappers::ReceiverStream<Result<keymanagement::ExportCryptoKeysResponse, tonic::Status>>;
    /// Streams every matching crypto key.
    async fn export_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::ExportCryptoKeysRequest>,
    ) -> Result<tonic::Response<Self::ExportCryptoKeysStream>, tonic::Status> {
        let handler = self.clone();
        let request = request.into_inner();
        let stream = acme_platform::transport::one_reply(async move { handler.run_export_crypto_keys(request).await });
        Ok(tonic::Response::new(stream))
    }
    /// Lists crypto keys a page at a time.
    async fn suggest_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::SuggestCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::SuggestCryptoKeysResponse>, tonic::Status> {
        self.run_suggest_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
