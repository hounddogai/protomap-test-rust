use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_query_service_client::AuditEventQueryServiceClient;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::broadcasts::v1::broadcast_service_client::BroadcastServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_query_service_client::CryptoKeyQueryServiceClient;
use acme_proto::acme::keymanagement::v1::crypto_key_service_server::CryptoKeyService;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_client::CryptoKeyValidationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::keymanagement::v1::key_ring_validation_service_client::KeyRingValidationServiceClient;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::metrics::v1::metric_series_validation_service_client::MetricSeriesValidationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::seating::v1 as seating;
use acme_proto::acme::seating::v1::seat_map_service_client::SeatMapServiceClient;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::shows::v1::show_service_client::ShowServiceClient;

/// Manages crypto keys: creates, reads, changes, and lists them.
#[derive(Clone)]
pub struct CryptoKeyHandler {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
}
impl CryptoKeyHandler {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run CreateCryptoKey after validating the tenant and durably recording the command.
    async fn run_create_crypto_key(
        &self,
        req: keymanagement::CreateCryptoKeyRequest,
    ) -> Result<keymanagement::CreateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CreateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CreateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().holder.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::PreviewKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    audit: Some(common::AuditInfo {
                        created_at: req
                            .crypto_key
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        ..Default::default()
                    }),
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            etag: req
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
                        ..Default::default()
                    }],
                    manager: Some(identity::UserRef {
                        email: req
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
        let jobs_1 = req
            .crypto_key
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = keymanagement::ValidateCryptoKeyRequest {
                        context: req.context.clone(),
                        crypto_key: Some(keymanagement::CryptoKey {
                            limits: vec![keymanagement::CryptoKeyLimit {
                                amount: item.amount.clone(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };
                    async move {
                        let mut client = CryptoKeyValidationServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::ValidateCryptoKeyRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.validate_crypto_key(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let reply_1 = replies.into_iter().last().unwrap_or_default();
        let outgoing_2 = metrics::ValidateMetricSeriesRequest {
            context: req.context.clone(),
            details: Some(metrics::validate_metric_series_request::Details {
                contact: Some(common::ContactInfo {
                    name: req
                        .crypto_key
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
            metric_series: Some(metrics::MetricSeries {
                locator: Some(metrics::metric_series::Locator::Address(common::PostalAddress {
                    country_code: reply_0
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .location
                        .clone()
                        .unwrap_or_default()
                        .country_code
                        .clone(),
                    ..Default::default()
                })),
                owner: Some(identity::UserRef {
                    user_id: reply_0
                        .key_ring
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
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = MetricSeriesValidationServiceClient::new(channel);
            let input: tonic::Request<metrics::ValidateMetricSeriesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_metric_series(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = keymanagement::SimulateKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: reply_2
                                .metric_series
                                .clone()
                                .unwrap_or_default()
                                .owner
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
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = KeyRingCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::SimulateKeyRingRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_key_ring(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_4 = seating::BatchCreateSeatMapsRequest {
            context: req.context.clone(),
            chunk: Some(seating::batch_create_seat_maps_request::Chunk::Record(
                seating::SeatMap {
                    children: vec![seating::SeatMap {
                        children: vec![seating::SeatMap {
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
                                updated_at: reply_3
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
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_4 = async move {
            let mut client = SeatMapServiceClient::new(channel);
            let input: tonic::Request<seating::BatchCreateSeatMapsRequest> =
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
            let response = client.batch_create_seat_maps(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_5 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    owner: Some(identity::UserRef {
                        email: req
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
        let mut response = keymanagement::CreateCryptoKeyResponse::default();
        let contribution: String = reply_0
            .key_ring
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .etag
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_0.key_ring.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            if response
                .crypto_key
                .get_or_insert_with(Default::default)
                .limits
                .is_empty()
            {
                response
                    .crypto_key
                    .get_or_insert_with(Default::default)
                    .limits
                    .push(Default::default());
            }
            response.crypto_key.get_or_insert_with(Default::default).limits[0].name = contribution;
        }
        let contribution: String = reply_2
            .metric_series
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_by
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = reply_3
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
                .crypto_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_3.breakdown.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = reply_4.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_5.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: i32 = req.crypto_key.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.crypto_key.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run GetCryptoKey after validating the tenant and durably recording the command.
    async fn run_get_crypto_key(
        &self,
        req: keymanagement::GetCryptoKeyRequest,
    ) -> Result<keymanagement::GetCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCryptoKey", "running workflow");
        let outgoing_0 = broadcasts::CreateBroadcastRequest {
            broadcast: Some(broadcasts::Broadcast {
                audit: Some(common::AuditInfo {
                    updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                organizer: Some(identity::UserRef {
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
                participants: vec![broadcasts::BroadcastParticipant {
                    participant_id: req.crypto_key_id.clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = BroadcastServiceClient::new(channel);
            let input: tonic::Request<broadcasts::CreateBroadcastRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_broadcast(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::ListFlagRolloutSubscriptionsRequest {
            context: req.context.clone(),
            filter: Some(featureflags::list_flag_rollout_subscriptions_request::Filter {
                created_after: reply_0
                    .broadcast
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                owner_id: reply_0
                    .broadcast
                    .clone()
                    .unwrap_or_default()
                    .organizer
                    .clone()
                    .unwrap_or_default()
                    .user_id
                    .clone(),
                region: reply_0
                    .broadcast
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
            let mut client = FlagRolloutNotificationServiceClient::new(channel);
            let input: tonic::Request<featureflags::ListFlagRolloutSubscriptionsRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.list_flag_rollout_subscriptions(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::GetCryptoKeyResponse::default();
        let contribution: String = reply_0
            .broadcast
            .clone()
            .unwrap_or_default()
            .organizer
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_0
            .broadcast
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        Ok(response)
    }
    /// Run UpdateCryptoKey after validating the tenant and durably recording the command.
    async fn run_update_crypto_key(
        &self,
        req: keymanagement::UpdateCryptoKeyRequest,
    ) -> Result<keymanagement::UpdateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("UpdateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "UpdateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().holder.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::GetCryptoKeyRatesRequest {
            context: req.context.clone(),
            crypto_key_id: req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CryptoKeyCalculationServiceClient::new(channel);
            let input: tonic::Request<keymanagement::GetCryptoKeyRatesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.get_crypto_key_rates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::UpdateCryptoKeyResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.crypto_key.clone().unwrap_or_default().labels.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .labels
                .extend(contribution);
        }
        let contribution: Option<common::Money> = req.crypto_key.clone().unwrap_or_default().balance.clone();
        if contribution != <Option<common::Money>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).balance = contribution;
        }
        let contribution: String = req.crypto_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run DeleteCryptoKey after validating the tenant and durably recording the command.
    async fn run_delete_crypto_key(
        &self,
        req: keymanagement::DeleteCryptoKeyRequest,
    ) -> Result<keymanagement::DeleteCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("DeleteCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "DeleteCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CalculateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            etag: req.etag.clone(),
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
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::CalculateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::DeleteCryptoKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run ListCryptoKeys after validating the tenant and durably recording the command.
    async fn run_list_crypto_keys(
        &self,
        req: keymanagement::ListCryptoKeysRequest,
    ) -> Result<keymanagement::ListCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ListCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = certificates::ResolveCertificateAuthoritiesRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateAuthorityQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ResolveCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_certificate_authorities(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::ListCryptoKeysResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        Ok(response)
    }
    /// Run BatchGetCryptoKeys after validating the tenant and durably recording the command.
    async fn run_batch_get_crypto_keys(
        &self,
        req: keymanagement::BatchGetCryptoKeysRequest,
    ) -> Result<keymanagement::BatchGetCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchGetCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchGetCryptoKeys", "running workflow");
        let outgoing_0 = ratelimiting::UpdateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = RateLimitServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::UpdateRateLimitRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_rate_limit(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::BatchGetCryptoKeysResponse::default();
        let contribution: String = reply_0
            .rate_limit
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
    /// Run SearchCryptoKeys after validating the tenant and durably recording the command.
    async fn run_search_crypto_keys(
        &self,
        req: keymanagement::SearchCryptoKeysRequest,
    ) -> Result<keymanagement::SearchCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SearchCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SearchCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CompareQuotaBucketOptionsRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::compare_quota_bucket_options_request::Details {
                destination: Some(common::PostalAddress {
                    region: req.filter.clone().unwrap_or_default().region.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                audit: Some(common::AuditInfo {
                    updated_at: req.filter.clone().unwrap_or_default().created_after.clone(),
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
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
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
        let mut response = keymanagement::SearchCryptoKeysResponse::default();
        let contribution: String = reply_0
            .quota_bucket
            .clone()
            .unwrap_or_default()
            .contact
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            if response.crypto_keys.is_empty() {
                response.crypto_keys.push(Default::default());
            }
            response.crypto_keys[0]
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
        }
        Ok(response)
    }
    /// Run WatchCryptoKeys after validating the tenant and durably recording the command.
    async fn run_watch_crypto_keys(
        &self,
        req: keymanagement::WatchCryptoKeysRequest,
    ) -> Result<keymanagement::WatchCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("WatchCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "WatchCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.subscriber.clone().unwrap_or_default().email.clone(), "recorded audit field");
        let outgoing_0 = audit::SuggestAuditEventsRequest {
            context: req.context.clone(),
            filter: Some(audit::suggest_audit_events_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                query: req.filter.clone().unwrap_or_default().query.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = keymanagement::BrowseCryptoKeysRequest {
            context: req.context.clone(),
            filter: Some(keymanagement::browse_crypto_keys_request::Filter {
                created_after: req.filter.clone().unwrap_or_default().created_after.clone(),
                labels: req.filter.clone().unwrap_or_default().labels.clone(),
                owner_id: req.filter.clone().unwrap_or_default().owner_id.clone(),
                region: req.filter.clone().unwrap_or_default().region.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = AuditEventQueryServiceClient::new(channel_0);
                let input: tonic::Request<audit::SuggestAuditEventsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.suggest_audit_events(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = CryptoKeyQueryServiceClient::new(channel_1);
                let input: tonic::Request<keymanagement::BrowseCryptoKeysRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.browse_crypto_keys(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = shows::BatchGetShowsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ShowServiceClient::new(channel);
            let input: tonic::Request<shows::BatchGetShowsRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_shows(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = ratelimiting::EstimateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
                contact: Some(common::ContactInfo {
                    phone: req.subscriber.clone().unwrap_or_default().phone.clone(),
                    preferred_channel: req.subscriber.clone().unwrap_or_default().preferred_channel.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::EstimateQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.estimate_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::WatchCryptoKeysResponse::default();
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
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
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: Option<common::PersonName> = req.subscriber.clone().unwrap_or_default().name.clone();
        if contribution != <Option<common::PersonName>>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .name = contribution;
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
    /// Run BatchCreateCryptoKeys after validating the tenant and durably recording the command.
    async fn run_batch_create_crypto_keys(
        &self,
        req: keymanagement::BatchCreateCryptoKeysRequest,
    ) -> Result<keymanagement::BatchCreateCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("BatchCreateCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "BatchCreateCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?(match req.chunk.clone() { Some(keymanagement::batch_create_crypto_keys_request::Chunk::Header(value)) => value, _ => Default::default() }).source.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::RecalculateKeyRingsRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_at: (match req.chunk.clone() {
                                Some(keymanagement::batch_create_crypto_keys_request::Chunk::Record(value)) => value,
                                _ => Default::default(),
                            })
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                            ..Default::default()
                        }),
                        children: vec![keymanagement::KeyRing {
                            key_ring_id: (match req.chunk.clone() {
                                Some(keymanagement::batch_create_crypto_keys_request::Chunk::Record(value)) => value,
                                _ => Default::default(),
                            })
                            .holder
                            .clone()
                            .unwrap_or_default()
                            .key_ring_id
                            .clone(),
                            ..Default::default()
                        }],
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
        let mut response = keymanagement::BatchCreateCryptoKeysResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run CloneCryptoKey after validating the tenant and durably recording the command.
    async fn run_clone_crypto_key(
        &self,
        req: keymanagement::CloneCryptoKeyRequest,
    ) -> Result<keymanagement::CloneCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CloneCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CloneCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::RecalculateKeyRingsRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            etag: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .etag
                                .clone(),
                            updated_by: req
                                .crypto_key
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
        let jobs_1 = req
            .crypto_key
            .clone()
            .unwrap_or_default()
            .limits
            .clone()
            .into_iter()
            .map(|item| {
                let req = req.clone();
                let channel = self.channel.clone();
                async move {
                    let outgoing_1 = configuration::SimulateConfigSetRequest {
                        config_set: Some(configuration::ConfigSet {
                            children: vec![configuration::ConfigSet {
                                children: vec![configuration::ConfigSet {
                                    children: vec![configuration::ConfigSet {
                                        name: item.name.clone(),
                                        ..Default::default()
                                    }],
                                    ..Default::default()
                                }],
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        context: req.context.clone(),
                        ..Default::default()
                    };
                    async move {
                        let mut client = ConfigSetCalculationServiceClient::new(channel);
                        let input: tonic::Request<configuration::SimulateConfigSetRequest> =
                            tonic::IntoRequest::into_request(outgoing_1);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.simulate_config_set(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_1).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_1 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::CloneCryptoKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
        }
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
                .crypto_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.crypto_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: Option<common::Money> = req.crypto_key.clone().unwrap_or_default().available.clone();
        if contribution != <Option<common::Money>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).available = contribution;
        }
        Ok(response)
    }
}
acme_platform::endpoints! { CryptoKeyService, CryptoKeyHandler;
unary {
create_crypto_key (keymanagement::CreateCryptoKeyRequest) -> keymanagement::CreateCryptoKeyResponse => run_create_crypto_key;
get_crypto_key (keymanagement::GetCryptoKeyRequest) -> keymanagement::GetCryptoKeyResponse => run_get_crypto_key;
update_crypto_key (keymanagement::UpdateCryptoKeyRequest) -> keymanagement::UpdateCryptoKeyResponse => run_update_crypto_key;
delete_crypto_key (keymanagement::DeleteCryptoKeyRequest) -> keymanagement::DeleteCryptoKeyResponse => run_delete_crypto_key;
list_crypto_keys (keymanagement::ListCryptoKeysRequest) -> keymanagement::ListCryptoKeysResponse => run_list_crypto_keys;
batch_get_crypto_keys (keymanagement::BatchGetCryptoKeysRequest) -> keymanagement::BatchGetCryptoKeysResponse => run_batch_get_crypto_keys;
search_crypto_keys (keymanagement::SearchCryptoKeysRequest) -> keymanagement::SearchCryptoKeysResponse => run_search_crypto_keys;
clone_crypto_key (keymanagement::CloneCryptoKeyRequest) -> keymanagement::CloneCryptoKeyResponse => run_clone_crypto_key;
}
server {
watch_crypto_keys (keymanagement::WatchCryptoKeysRequest) -> keymanagement::WatchCryptoKeysResponse, WatchCryptoKeysStream => run_watch_crypto_keys;
}
client {
batch_create_crypto_keys (keymanagement::BatchCreateCryptoKeysRequest) -> keymanagement::BatchCreateCryptoKeysResponse => run_batch_create_crypto_keys;
}
bidi {
}
}
