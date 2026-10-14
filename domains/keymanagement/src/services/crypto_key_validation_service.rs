use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_history_service_client::RoleBindingHistoryServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::certificates::v1::certificate_service_client::CertificateServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_validation_service_server::CryptoKeyValidationService;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::throttle_validation_service_client::ThrottleValidationServiceClient;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::sponsorships::v1::sponsor_calculation_service_client::SponsorCalculationServiceClient;
use acme_proto::acme::sponsorships::v1::sponsor_service_client::SponsorServiceClient;

/// Checks crypto keys against the rules before anything acts on them.
#[derive(Clone)]
pub struct CryptoKeyValidationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    get_crypto_key_rules_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<keymanagement::QuoteKeyRingRequest, keymanagement::QuoteKeyRingResponse>,
    >,
    lint_crypto_keys_0_port:
        acme_platform::ports::Callback<keymanagement::EstimateKeyRingRequest, keymanagement::EstimateKeyRingResponse>,
}
impl CryptoKeyValidationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            keymanagement::EstimateKeyRingRequest,
            keymanagement::EstimateKeyRingResponse,
        > = {
            let client = KeyRingCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::EstimateKeyRingRequest| -> acme_platform::ports::RpcFuture<keymanagement::EstimateKeyRingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.estimate_key_ring(request).await?.into_inner())
})
})
        };
        let lint_crypto_keys_0_port = callback;
        let callback: acme_platform::ports::Callback<
            keymanagement::QuoteKeyRingRequest,
            keymanagement::QuoteKeyRingResponse,
        > = {
            let client = KeyRingCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: keymanagement::QuoteKeyRingRequest| -> acme_platform::ports::RpcFuture<keymanagement::QuoteKeyRingResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.quote_key_ring(request).await?.into_inner())
})
})
        };
        let get_crypto_key_rules_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
        Self {
            channel,
            repository,
            get_crypto_key_rules_0_port,
            lint_crypto_keys_0_port,
        }
    }
    /// Run ValidateCryptoKey after validating the tenant and durably recording the command.
    async fn run_validate_crypto_key(
        &self,
        req: keymanagement::ValidateCryptoKeyRequest,
    ) -> Result<keymanagement::ValidateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ValidateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ValidateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = sponsorships::QuoteSponsorRequest {
            context: req.context.clone(),
            sponsor: Some(sponsorships::Sponsor {
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
                    updated_at: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }),
                contact: Some(common::ContactInfo {
                    preferred_channel: req
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
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<sponsorships::QuoteSponsorRequest>(4);
        let channel = self.channel.clone();
        let worker = tokio::spawn(async move {
            let pending = receiver
                .recv()
                .await
                .ok_or_else(|| tonic::Status::cancelled("request cancelled"))?;
            async move {
                let mut client = SponsorCalculationServiceClient::new(channel);
                let input: tonic::Request<sponsorships::QuoteSponsorRequest> =
                    tonic::IntoRequest::into_request(pending);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.quote_sponsor(input).await?;
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
        let outgoing_1 = certificates::UpdateCertificateRequest {
            certificate: Some(certificates::Certificate {
                billing_address: Some(common::PostalAddress {
                    postal_code: reply_0
                        .sponsor
                        .clone()
                        .unwrap_or_default()
                        .primary_address
                        .clone()
                        .unwrap_or_default()
                        .postal_code
                        .clone(),
                    ..Default::default()
                }),
                settlement: Some(certificates::certificate::Settlement::Card(
                    certificates::CertificateCardSettlement {
                        holder_name: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .holder_name
                        .clone(),
                        network: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Card(value)) => value,
                            _ => Default::default(),
                        })
                        .network
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = acme_platform::transport::deadline(async move {
            let mut client = CertificateServiceClient::new(channel);
            let input: tonic::Request<certificates::UpdateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.update_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let outgoing_2 = sponsorships::BatchCreateSponsorsRequest {
            context: req.context.clone(),
            chunk: Some(sponsorships::batch_create_sponsors_request::Chunk::Record(
                sponsorships::Sponsor {
                    audit: Some(common::AuditInfo {
                        created_at: reply_0
                            .sponsor
                            .clone()
                            .unwrap_or_default()
                            .audit
                            .clone()
                            .unwrap_or_default()
                            .created_at
                            .clone(),
                        ..Default::default()
                    }),
                    primary_address: Some(common::PostalAddress {
                        locality: reply_1
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
                },
            )),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = SponsorServiceClient::new(channel);
            let input: tonic::Request<sponsorships::BatchCreateSponsorsRequest> =
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
            let response = client.batch_create_sponsors(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagEvaluationChangesRequest {
            frame: Some(featureflags::stream_flag_evaluation_changes_request::Frame::Change(
                featureflags::FlagEvaluation {
                    subject: Some(featureflags::FlagRolloutRef {
                        display_name: reply_0
                            .sponsor
                            .clone()
                            .unwrap_or_default()
                            .relationship_manager
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
        let reply_3 = async move {
            let mut client = FlagEvaluationSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagEvaluationChangesRequest> =
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
        let mut response = keymanagement::ValidateCryptoKeyResponse::default();
        let contribution: String = reply_0
            .sponsor
            .clone()
            .unwrap_or_default()
            .relationship_manager
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: i32 = req.crypto_key.clone().unwrap_or_default().status.clone();
        if contribution != <i32>::default() {
            response.crypto_key.get_or_insert_with(Default::default).status = contribution;
        }
        Ok(response)
    }
    /// Run PreviewCryptoKey after validating the tenant and durably recording the command.
    async fn run_preview_crypto_key(
        &self,
        req: keymanagement::PreviewCryptoKeyRequest,
    ) -> Result<keymanagement::PreviewCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PreviewCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PreviewCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = featureflags::ResolveFlagRolloutConflictRequest {
            context: req.context.clone(),
            details: Some(featureflags::resolve_flag_rollout_conflict_request::Details {
                payload: acme_platform::codec::json_payload(acme_platform::codec::AuditEnvelope {
                    trace_id: req.context.clone().unwrap_or_default().trace_id.clone(),
                    tenant_id: req.context.clone().unwrap_or_default().tenant_id.clone(),
                })?,
                ..Default::default()
            }),
            flag_rollout: Some(featureflags::FlagRollout {
                participants: vec![featureflags::FlagRolloutParticipant {
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
                        name: req
                            .crypto_key
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .name
                            .clone(),
                        phone: req
                            .crypto_key
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
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::ResolveFlagRolloutConflictRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_flag_rollout_conflict(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::PreviewCryptoKeyResponse::default();
        let contribution: String = reply_0.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<common::ContactInfo> = req.details.clone().unwrap_or_default().contact.clone();
        if contribution != <Option<common::ContactInfo>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).contact = contribution;
        }
        Ok(response)
    }
    /// Run ExplainCryptoKeyDecision after validating the tenant and durably recording the command.
    async fn run_explain_crypto_key_decision(
        &self,
        req: keymanagement::ExplainCryptoKeyDecisionRequest,
    ) -> Result<keymanagement::ExplainCryptoKeyDecisionResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainCryptoKeyDecision", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainCryptoKeyDecision", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::ScoreThrottleRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            throttle: Some(ratelimiting::Throttle {
                subject: Some(ratelimiting::RateLimitRef {
                    display_name: req
                        .crypto_key
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
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = authorization::CompareRoleBindingVersionsRequest {
            context: req.context.clone(),
            role_binding: Some(authorization::RoleBinding {
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
                    ..Default::default()
                }),
                settlement: Some(authorization::role_binding::Settlement::Bank(
                    authorization::RoleBindingBankSettlement {
                        iban: (match req.crypto_key.clone().unwrap_or_default().funding.clone() {
                            Some(keymanagement::crypto_key::Funding::Bank(value)) => value,
                            _ => Default::default(),
                        })
                        .iban
                        .clone(),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ThrottleValidationServiceClient::new(channel_0);
                let input: tonic::Request<ratelimiting::ScoreThrottleRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.score_throttle(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = RoleBindingHistoryServiceClient::new(channel_1);
                let input: tonic::Request<authorization::CompareRoleBindingVersionsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.compare_role_binding_versions(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = keymanagement::ExplainCryptoKeyDecisionResponse::default();
        let contribution: String = reply_0.throttle.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
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
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: String = req.crypto_key.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run ScoreCryptoKey after validating the tenant and durably recording the command.
    async fn run_score_crypto_key(
        &self,
        req: keymanagement::ScoreCryptoKeyRequest,
    ) -> Result<keymanagement::ScoreCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ScoreCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ScoreCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let jobs_0 = req
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
                    let outgoing_0 = keymanagement::SimulateKeyRingRequest {
                        context: req.context.clone(),
                        key_ring: Some(keymanagement::KeyRing {
                            children: vec![keymanagement::KeyRing {
                                children: vec![keymanagement::KeyRing {
                                    audit: Some(common::AuditInfo {
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
                                    children: vec![keymanagement::KeyRing {
                                        name: item.name.clone(),
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
                    async move {
                        let mut client = KeyRingCalculationServiceClient::new(channel);
                        let input: tonic::Request<keymanagement::SimulateKeyRingRequest> =
                            tonic::IntoRequest::into_request(outgoing_0);
                        let context = input.get_ref().context.clone().unwrap_or_default();
                        let input = acme_platform::auth::service_request(input, &context)?;
                        let response = client.simulate_key_ring(input).await?;
                        Ok::<_, tonic::Status>(response.into_inner())
                    }
                    .await
                }
            })
            .collect::<Vec<_>>();
        let replies = futures::future::join_all(jobs_0).await;
        let replies = replies.into_iter().collect::<Result<Vec<_>, tonic::Status>>()?;
        let _reply_0 = replies.into_iter().last().unwrap_or_default();
        let mut response = keymanagement::ScoreCryptoKeyResponse::default();
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: String = req.details.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.breakdown.get_or_insert_with(Default::default).currency_code = contribution;
        }
        Ok(response)
    }
    /// Run LintCryptoKeys after validating the tenant and durably recording the command.
    async fn run_lint_crypto_keys(
        &self,
        req: keymanagement::LintCryptoKeysRequest,
    ) -> Result<keymanagement::LintCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("LintCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "LintCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = keymanagement::EstimateKeyRingRequest {
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
                            updated_at: req
                                .crypto_key
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_at
                                .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .crypto_key
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let port = acme_platform::ports::opaque(self.lint_crypto_keys_0_port.clone());
        let reply_0 = acme_platform::ports::execute(&port, outgoing_0).await?;
        let outgoing_1 = certificates::CalculateCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    updated_by: reply_0
                        .key_ring
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_by
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
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
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<certificates::CalculateCertificateRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::CalculateCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.calculate_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::LintCryptoKeysResponse::default();
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
        }
        let contribution: String = reply_1
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
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_by = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<common::Money> = req.crypto_key.clone().unwrap_or_default().available.clone();
        if contribution != <Option<common::Money>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).available = contribution;
        }
        Ok(response)
    }
    /// Run GetCryptoKeyRules after validating the tenant and durably recording the command.
    async fn run_get_crypto_key_rules(
        &self,
        req: keymanagement::GetCryptoKeyRulesRequest,
    ) -> Result<keymanagement::GetCryptoKeyRulesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCryptoKeyRules", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCryptoKeyRules", "running workflow");
        let outgoing_0 = keymanagement::QuoteKeyRingRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        children: vec![keymanagement::KeyRing {
                            parent_id: req.crypto_key_id.clone(),
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
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
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
            }),
            ..Default::default()
        };
        let reply_0 =
            self.get_crypto_key_rules_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = keymanagement::GetCryptoKeyRulesResponse::default();
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
        let contribution: String = reply_0
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
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_by = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        Ok(response)
    }
}
#[tonic::async_trait]
impl CryptoKeyValidationService for CryptoKeyValidationController {
    /// Validates a crypto key.
    async fn validate_crypto_key(
        &self,
        request: tonic::Request<keymanagement::ValidateCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::ValidateCryptoKeyResponse>, tonic::Status> {
        self.run_validate_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Previews a crypto key.
    async fn preview_crypto_key(
        &self,
        request: tonic::Request<keymanagement::PreviewCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::PreviewCryptoKeyResponse>, tonic::Status> {
        self.run_preview_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a crypto key.
    async fn explain_crypto_key_decision(
        &self,
        request: tonic::Request<keymanagement::ExplainCryptoKeyDecisionRequest>,
    ) -> Result<tonic::Response<keymanagement::ExplainCryptoKeyDecisionResponse>, tonic::Status> {
        self.run_explain_crypto_key_decision(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Scores a crypto key.
    async fn score_crypto_key(
        &self,
        request: tonic::Request<keymanagement::ScoreCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::ScoreCryptoKeyResponse>, tonic::Status> {
        self.run_score_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Lints a crypto key.
    async fn lint_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::LintCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::LintCryptoKeysResponse>, tonic::Status> {
        self.run_lint_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a crypto key.
    async fn get_crypto_key_rules(
        &self,
        request: tonic::Request<keymanagement::GetCryptoKeyRulesRequest>,
    ) -> Result<tonic::Response<keymanagement::GetCryptoKeyRulesResponse>, tonic::Status> {
        self.run_get_crypto_key_rules(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
