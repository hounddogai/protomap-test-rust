use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::audit::v1::audit_event_service_client::AuditEventServiceClient;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::boxoffice::v1::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::broadcasts::v1::broadcast_service_client::BroadcastServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient;
use acme_proto::acme::certificates::v1::certificate_query_service_client::CertificateQueryServiceClient;
use acme_proto::acme::certificates::v1::csr_validation_service_client::CsrValidationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_sync_service_client::FeatureFlagSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::identity::v1::user_query_service_client::UserQueryServiceClient;
use acme_proto::acme::identity::v1::user_service_client::UserServiceClient;
use acme_proto::acme::identity::v1::user_validation_service_client::UserValidationServiceClient;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::crypto_key_calculation_service_server::CryptoKeyCalculationService;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::notifications::v1::notification_validation_service_client::NotificationValidationServiceClient;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::patients::v1::patient_service_client::PatientServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1::rate_limit_service_client::RateLimitServiceClient;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::sponsorships::v1::sponsor_service_client::SponsorServiceClient;
use acme_proto::acme::venues::v1 as venues;
use acme_proto::acme::venues::v1::venue_query_service_client::VenueQueryServiceClient;

/// Computes the figures of a crypto key, such as totals, taxes, and estimates.
#[derive(Clone)]
pub struct CryptoKeyCalculationController {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    explain_crypto_key_calculation_0_port: std::sync::Arc<
        acme_platform::ports::Adapter<ratelimiting::UpdateRateLimitRequest, ratelimiting::UpdateRateLimitResponse>,
    >,
    explain_crypto_key_calculation_1_port: std::sync::Arc<
        dyn acme_platform::ports::RpcPort<sponsorships::ListSponsorsRequest, sponsorships::ListSponsorsResponse>,
    >,
    explain_crypto_key_calculation_2_port:
        acme_platform::ports::Callback<identity::LookupUserRequest, identity::LookupUserResponse>,
    quote_crypto_key_0_port: std::collections::HashMap<
        &'static str,
        acme_platform::ports::Callback<
            ratelimiting::CalculateQuotaBucketRequest,
            ratelimiting::CalculateQuotaBucketResponse,
        >,
    >,
}
impl CryptoKeyCalculationController {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::KeymanagementRepository>,
    ) -> Self {
        let callback: acme_platform::ports::Callback<
            ratelimiting::CalculateQuotaBucketRequest,
            ratelimiting::CalculateQuotaBucketResponse,
        > = {
            let client = QuotaBucketCalculationServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: ratelimiting::CalculateQuotaBucketRequest| -> acme_platform::ports::RpcFuture<ratelimiting::CalculateQuotaBucketResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.calculate_quota_bucket(request).await?.into_inner())
})
})
        };
        let quote_crypto_key_0_port =
            std::collections::HashMap::from([("primary", callback.clone()), ("replay", callback)]);
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
        let explain_crypto_key_calculation_0_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<
            sponsorships::ListSponsorsRequest,
            sponsorships::ListSponsorsResponse,
        > = {
            let client = SponsorServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: sponsorships::ListSponsorsRequest| -> acme_platform::ports::RpcFuture<sponsorships::ListSponsorsResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.list_sponsors(request).await?.into_inner())
})
})
        };
        let explain_crypto_key_calculation_1_port = std::sync::Arc::new(acme_platform::ports::Adapter::new(callback));
        let callback: acme_platform::ports::Callback<identity::LookupUserRequest, identity::LookupUserResponse> = {
            let client = UserQueryServiceClient::new(channel.clone());
            std::sync::Arc::new(move |request: identity::LookupUserRequest| -> acme_platform::ports::RpcFuture<identity::LookupUserResponse> {
let mut client = client.clone();
Box::pin(async move {
let context = request.context.clone().unwrap_or_default();
let request = acme_platform::auth::service_request(tonic::Request::new(request), &context)?;
Ok(client.lookup_user(request).await?.into_inner())
})
})
        };
        let explain_crypto_key_calculation_2_port = callback;
        Self {
            channel,
            repository,
            explain_crypto_key_calculation_0_port,
            explain_crypto_key_calculation_1_port,
            explain_crypto_key_calculation_2_port,
            quote_crypto_key_0_port,
        }
    }
    /// Run CalculateCryptoKey after validating the tenant and durably recording the command.
    async fn run_calculate_crypto_key(
        &self,
        req: keymanagement::CalculateCryptoKeyRequest,
    ) -> Result<keymanagement::CalculateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("CalculateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CalculateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = identity::ListUsersRequest {
            context: req.context.clone(),
            filter: Some(identity::list_users_request::Filter {
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = identity::PeekUserRequest {
            context: req.context.clone(),
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
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = UserServiceClient::new(channel_0);
                let input: tonic::Request<identity::ListUsersRequest> = tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_users(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = UserQueryServiceClient::new(channel_1);
                let input: tonic::Request<identity::PeekUserRequest> = tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.peek_user(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = patients::BatchCreatePatientsRequest {
            context: req.context.clone(),
            chunk: Some(patients::batch_create_patients_request::Chunk::Record(
                patients::Patient {
                    audit: Some(common::AuditInfo {
                        created_at: reply_1
                            .user
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
                        email: reply_1
                            .user
                            .clone()
                            .unwrap_or_default()
                            .contact
                            .clone()
                            .unwrap_or_default()
                            .email
                            .clone(),
                        ..Default::default()
                    }),
                    identity_document: Some(patients::patient::IdentityDocument::Passport(
                        patients::PatientPassport {
                            number: (match reply_1.user.clone().unwrap_or_default().identity_document.clone() {
                                Some(identity::user::IdentityDocument::Passport(value)) => value,
                                _ => Default::default(),
                            })
                            .number
                            .clone(),
                            ..Default::default()
                        },
                    )),
                    preferences: Some(patients::PatientPreferences {
                        channels: reply_1
                            .user
                            .clone()
                            .unwrap_or_default()
                            .preferences
                            .clone()
                            .unwrap_or_default()
                            .channels
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
            let mut client = PatientServiceClient::new(channel);
            let input: tonic::Request<patients::BatchCreatePatientsRequest> =
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
            let response = client.batch_create_patients(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFeatureFlagChangesRequest {
            frame: Some(featureflags::stream_feature_flag_changes_request::Frame::Change(
                featureflags::FeatureFlag {
                    owner: Some(identity::UserRef {
                        display_name: req
                            .crypto_key
                            .clone()
                            .unwrap_or_default()
                            .holder
                            .clone()
                            .unwrap_or_default()
                            .display_name
                            .clone(),
                        user_id: reply_1
                            .user
                            .clone()
                            .unwrap_or_default()
                            .relationship_manager
                            .clone()
                            .unwrap_or_default()
                            .user_id
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
        let mut response = keymanagement::CalculateCryptoKeyResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1
            .user
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
        let contribution: String = reply_1
            .user
            .clone()
            .unwrap_or_default()
            .relationship_manager
            .clone()
            .unwrap_or_default()
            .email
            .clone();
        if contribution != <String>::default() {
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .contact
                .get_or_insert_with(Default::default)
                .email = contribution;
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
    /// Run EstimateCryptoKey after validating the tenant and durably recording the command.
    async fn run_estimate_crypto_key(
        &self,
        req: keymanagement::EstimateCryptoKeyRequest,
    ) -> Result<keymanagement::EstimateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("EstimateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "EstimateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().holder.clone().unwrap_or_default().key_ring_id.clone(), "recorded audit field");
        let outgoing_0 = venues::FindVenuesRequest {
            context: req.context.clone(),
            filter: Some(venues::find_venues_request::Filter {
                created_after: req.crypto_key.clone().unwrap_or_default().opened_at.clone(),
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
                owner_id: req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(),
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = broadcasts::BatchGetBroadcastsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = VenueQueryServiceClient::new(channel_0);
                let input: tonic::Request<venues::FindVenuesRequest> = tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.find_venues(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = BroadcastServiceClient::new(channel_1);
                let input: tonic::Request<broadcasts::BatchGetBroadcastsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.batch_get_broadcasts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = keymanagement::EstimateCryptoKeyResponse::default();
        let contribution: std::collections::BTreeMap<String, String> =
            reply_0.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
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
    /// Run QuoteCryptoKey after validating the tenant and durably recording the command.
    async fn run_quote_crypto_key(
        &self,
        req: keymanagement::QuoteCryptoKeyRequest,
    ) -> Result<keymanagement::QuoteCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("QuoteCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "QuoteCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::CalculateQuotaBucketRequest {
            context: req.context.clone(),
            quota_bucket: Some(ratelimiting::QuotaBucket {
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
        let reply_0 =
            self.quote_crypto_key_0_port
                .get("primary")
                .ok_or_else(|| tonic::Status::failed_precondition("dispatch route is missing"))?(outgoing_0)
            .await?;
        let mut response = keymanagement::QuoteCryptoKeyResponse::default();
        let contribution: String = reply_0.quota_bucket.clone().unwrap_or_default().currency_code.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).currency_code = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.crypto_key.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run CompareCryptoKeyOptions after validating the tenant and durably recording the command.
    async fn run_compare_crypto_key_options(
        &self,
        req: keymanagement::CompareCryptoKeyOptionsRequest,
    ) -> Result<keymanagement::CompareCryptoKeyOptionsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("CompareCryptoKeyOptions", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "CompareCryptoKeyOptions", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = broadcasts::ListBroadcastsRequest {
            context: req.context.clone(),
            filter: Some(broadcasts::list_broadcasts_request::Filter {
                created_after: req.crypto_key.clone().unwrap_or_default().opened_at.clone(),
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = notifications::PreviewNotificationRequest {
            context: req.context.clone(),
            notification: Some(notifications::Notification {
                requester: Some(common::ContactInfo {
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = BroadcastServiceClient::new(channel_0);
                let input: tonic::Request<broadcasts::ListBroadcastsRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.list_broadcasts(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = NotificationValidationServiceClient::new(channel_1);
                let input: tonic::Request<notifications::PreviewNotificationRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.preview_notification(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = boxoffice::SummarizeBoxOfficeSalesRequest {
            context: req.context.clone(),
            reason: req.reason.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = acme_platform::transport::deadline(async move {
            let mut client = BoxOfficeSaleQueryServiceClient::new(channel);
            let input: tonic::Request<boxoffice::SummarizeBoxOfficeSalesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_box_office_sales(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        })
        .await?;
        let mut response = keymanagement::CompareCryptoKeyOptionsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).message = contribution;
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
    /// Run RecalculateCryptoKeys after validating the tenant and durably recording the command.
    async fn run_recalculate_crypto_keys(
        &self,
        req: keymanagement::RecalculateCryptoKeysRequest,
    ) -> Result<keymanagement::RecalculateCryptoKeysResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("RecalculateCryptoKeys", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "RecalculateCryptoKeys", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = certificates::ExportCertificateAuthoritiesRequest {
            context: req.context.clone(),
            filter: Some(certificates::export_certificate_authorities_request::Filter {
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
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
        let _reply_0 = async move {
            let mut client = CertificateAuthorityQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ExportCertificateAuthoritiesRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
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
        let outgoing_1 = audit::CloneAuditEventRequest {
            audit_event: Some(audit::AuditEvent {
                measurements: vec![audit::AuditEventMeasurement {
                    observed_at: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
                        .clone(),
                    ..Default::default()
                }],
                occurred_at: req.crypto_key.clone().unwrap_or_default().opened_at.clone(),
                subject: Some(audit::AuditTrailRef {
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
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_1 = acme_platform::ports::RequestBuilder::<audit::CloneAuditEventRequest>::default()
            .map(|draft| *draft = outgoing_1)
            .deadline(std::time::Duration::from_secs(8))
            .build();
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = AuditEventServiceClient::new(channel);
            let input: tonic::Request<audit::CloneAuditEventRequest> = tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.clone_audit_event(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let command_2 = RecalculateCryptoKeysDispatch2::from(req.clone());
        let outgoing_2: certificates::ResolveCertificatesRequest = command_2.into();
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CertificateQueryServiceClient::new(channel);
            let input: tonic::Request<certificates::ResolveCertificatesRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.resolve_certificates(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = venues::SummarizeVenuesRequest {
            annotations: req.annotations.clone(),
            context: req.context.clone(),
            ..Default::default()
        };
        let outgoing_3 = acme_platform::codec::through_any(&outgoing_3, "acme.venues.v1.SummarizeVenuesRequest")?;
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = VenueQueryServiceClient::new(channel);
            let input: tonic::Request<venues::SummarizeVenuesRequest> = tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.summarize_venues(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::RecalculateCryptoKeysResponse::default();
        let contribution: String = reply_1.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: std::collections::BTreeMap<String, String> =
            reply_3.result.clone().unwrap_or_default().details.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .details
                .extend(contribution);
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = req.crypto_key.clone().unwrap_or_default().opened_at.clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).opened_at = contribution;
        }
        Ok(response)
    }
    /// Run SimulateCryptoKey after validating the tenant and durably recording the command.
    async fn run_simulate_crypto_key(
        &self,
        req: keymanagement::SimulateCryptoKeyRequest,
    ) -> Result<keymanagement::SimulateCryptoKeyResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SimulateCryptoKey", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SimulateCryptoKey", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = identity::NormalizeUserRequest {
            context: req.context.clone(),
            user: Some(identity::User {
                primary_address: Some(common::PostalAddress {
                    postal_code: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = identity::LintUsersRequest {
            context: req.context.clone(),
            details: Some(identity::lint_users_request::Details {
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
            user: Some(identity::User {
                primary_address: Some(common::PostalAddress {
                    lines: req
                        .details
                        .clone()
                        .unwrap_or_default()
                        .destination
                        .clone()
                        .unwrap_or_default()
                        .lines
                        .clone(),
                    ..Default::default()
                }),
                relationship_manager: Some(identity::UserRef {
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
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = UserValidationServiceClient::new(channel_0);
                let input: tonic::Request<identity::NormalizeUserRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.normalize_user(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = UserValidationServiceClient::new(channel_1);
                let input: tonic::Request<identity::LintUsersRequest> = tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.lint_users(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let outgoing_2 = certificates::NormalizeCsrRequest {
            context: req.context.clone(),
            csr: Some(certificates::Csr {
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
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = CsrValidationServiceClient::new(channel);
            let input: tonic::Request<certificates::NormalizeCsrRequest> = tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.normalize_csr(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = keymanagement::SimulateCryptoKeyResponse::default();
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
            .user
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_2
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
                .crypto_key
                .get_or_insert_with(Default::default)
                .holder
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
    /// Run ExplainCryptoKeyCalculation after validating the tenant and durably recording the command.
    async fn run_explain_crypto_key_calculation(
        &self,
        req: keymanagement::ExplainCryptoKeyCalculationRequest,
    ) -> Result<keymanagement::ExplainCryptoKeyCalculationResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ExplainCryptoKeyCalculation", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ExplainCryptoKeyCalculation", "running workflow");
        tracing::debug!(audit_value = ?req.crypto_key.clone().unwrap_or_default().crypto_key_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::UpdateRateLimitRequest {
            context: req.context.clone(),
            rate_limit: Some(ratelimiting::RateLimit {
                audit: Some(common::AuditInfo {
                    updated_at: req
                        .crypto_key
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .updated_at
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
            }),
            ..Default::default()
        };
        let reply_0 =
            acme_platform::ports::execute(self.explain_crypto_key_calculation_0_port.as_ref(), outgoing_0).await?;
        let outgoing_1 = sponsorships::ListSponsorsRequest {
            context: req.context.clone(),
            filter: Some(sponsorships::list_sponsors_request::Filter {
                created_after: reply_0
                    .rate_limit
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .created_at
                    .clone(),
                labels: req.crypto_key.clone().unwrap_or_default().labels.clone(),
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
        let reply_1 =
            acme_platform::ports::execute(self.explain_crypto_key_calculation_1_port.as_ref(), outgoing_1).await?;
        let outgoing_2 = identity::LookupUserRequest {
            context: req.context.clone(),
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
        };
        let reply_2 = (self.explain_crypto_key_calculation_2_port)(outgoing_2).await?;
        let mut response = keymanagement::ExplainCryptoKeyCalculationResponse::default();
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
            response
                .crypto_key
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .etag = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
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
        let contribution: Option<keymanagement::KeyRingRef> = req.crypto_key.clone().unwrap_or_default().holder.clone();
        if contribution != <Option<keymanagement::KeyRingRef>>::default() {
            response.crypto_key.get_or_insert_with(Default::default).holder = contribution;
        }
        Ok(response)
    }
    /// Run GetCryptoKeyRates after validating the tenant and durably recording the command.
    async fn run_get_crypto_key_rates(
        &self,
        req: keymanagement::GetCryptoKeyRatesRequest,
    ) -> Result<keymanagement::GetCryptoKeyRatesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("GetCryptoKeyRates", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetCryptoKeyRates", "running workflow");
        let mut response = keymanagement::GetCryptoKeyRatesResponse::default();
        let contribution: String = req.crypto_key_id.clone();
        if contribution != <String>::default() {
            response.crypto_key.get_or_insert_with(Default::default).crypto_key_id = contribution;
        }
        Ok(response)
    }
}
/// The domain command prepared by RecalculateCryptoKeys for ResolveCertificates.
struct RecalculateCryptoKeysDispatch2 {
    context: Option<common::RequestContext>,
}
impl From<keymanagement::RecalculateCryptoKeysRequest> for RecalculateCryptoKeysDispatch2 {
    fn from(req: keymanagement::RecalculateCryptoKeysRequest) -> Self {
        Self {
            context: req.context.clone(),
        }
    }
}
impl From<RecalculateCryptoKeysDispatch2> for certificates::ResolveCertificatesRequest {
    fn from(value: RecalculateCryptoKeysDispatch2) -> Self {
        certificates::ResolveCertificatesRequest {
            context: value.context,
            ..Default::default()
        }
    }
}
#[tonic::async_trait]
impl CryptoKeyCalculationService for CryptoKeyCalculationController {
    /// Calculates a crypto key.
    async fn calculate_crypto_key(
        &self,
        request: tonic::Request<keymanagement::CalculateCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::CalculateCryptoKeyResponse>, tonic::Status> {
        self.run_calculate_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Estimates a crypto key.
    async fn estimate_crypto_key(
        &self,
        request: tonic::Request<keymanagement::EstimateCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::EstimateCryptoKeyResponse>, tonic::Status> {
        self.run_estimate_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Quotes a crypto key.
    async fn quote_crypto_key(
        &self,
        request: tonic::Request<keymanagement::QuoteCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::QuoteCryptoKeyResponse>, tonic::Status> {
        self.run_quote_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Compares a crypto key.
    async fn compare_crypto_key_options(
        &self,
        request: tonic::Request<keymanagement::CompareCryptoKeyOptionsRequest>,
    ) -> Result<tonic::Response<keymanagement::CompareCryptoKeyOptionsResponse>, tonic::Status> {
        self.run_compare_crypto_key_options(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Recalculates a crypto key.
    async fn recalculate_crypto_keys(
        &self,
        request: tonic::Request<keymanagement::RecalculateCryptoKeysRequest>,
    ) -> Result<tonic::Response<keymanagement::RecalculateCryptoKeysResponse>, tonic::Status> {
        self.run_recalculate_crypto_keys(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Simulates a crypto key.
    async fn simulate_crypto_key(
        &self,
        request: tonic::Request<keymanagement::SimulateCryptoKeyRequest>,
    ) -> Result<tonic::Response<keymanagement::SimulateCryptoKeyResponse>, tonic::Status> {
        self.run_simulate_crypto_key(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Explains a crypto key.
    async fn explain_crypto_key_calculation(
        &self,
        request: tonic::Request<keymanagement::ExplainCryptoKeyCalculationRequest>,
    ) -> Result<tonic::Response<keymanagement::ExplainCryptoKeyCalculationResponse>, tonic::Status> {
        self.run_explain_crypto_key_calculation(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
    /// Returns a crypto key.
    async fn get_crypto_key_rates(
        &self,
        request: tonic::Request<keymanagement::GetCryptoKeyRatesRequest>,
    ) -> Result<tonic::Response<keymanagement::GetCryptoKeyRatesResponse>, tonic::Status> {
        self.run_get_crypto_key_rates(request.into_inner())
            .await
            .map(tonic::Response::new)
    }
}
