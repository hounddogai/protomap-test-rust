use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::role_binding_validation_service_client::RoleBindingValidationServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_calculation_service_client::CertificateCalculationServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_calculation_service_client::ConfigChangeCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::configuration::v1::config_set_calculation_service_client::ConfigSetCalculationServiceClient;
use acme_proto::acme::configuration::v1::config_set_reporting_service_client::ConfigSetReportingServiceClient;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::configuration::v1::config_set_validation_service_client::ConfigSetValidationServiceClient;
use acme_proto::acme::configuration::v1::environment_sync_service_server::EnvironmentSyncService;
use acme_proto::acme::configuration::v1::environment_validation_service_client::EnvironmentValidationServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::flag_rollout_export_service_client::FlagRolloutExportServiceClient;
use acme_proto::acme::featureflags::v1::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::keymanagement::v1::key_ring_calculation_service_client::KeyRingCalculationServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient;
use acme_proto::acme::sellers::v1 as sellers;
use acme_proto::acme::sellers::v1::seller_service_client::SellerServiceClient;

/// Keeps environments in step with the systems of record in other regions.
#[derive(Clone)]
pub struct EnvironmentSyncEndpoint {
    pub channel: tonic::transport::Channel,
    repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
}
impl EnvironmentSyncEndpoint {
    pub fn new(
        channel: tonic::transport::Channel,
        repository: std::sync::Arc<crate::repository::ConfigurationRepository>,
    ) -> Self {
        Self { channel, repository }
    }
    /// Run SyncEnvironments after validating the tenant and durably recording the command.
    async fn run_sync_environments(
        &self,
        req: configuration::SyncEnvironmentsRequest,
    ) -> Result<configuration::SyncEnvironmentsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("SyncEnvironments", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "SyncEnvironments", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainConfigSetCalculationRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    audit: Some(common::AuditInfo {
                        updated_at: req
                            .environment
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
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainConfigSetCalculationRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_config_set_calculation(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = keymanagement::ExplainKeyRingCalculationRequest {
            context: req.context.clone(),
            key_ring: Some(keymanagement::KeyRing {
                children: vec![keymanagement::KeyRing {
                    children: vec![keymanagement::KeyRing {
                        audit: Some(common::AuditInfo {
                            updated_by: reply_0
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .updated_by
                                .clone(),
                            ..Default::default()
                        }),
                        location: Some(common::PostalAddress {
                            recipient: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .recipient
                            .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: reply_0
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .manager
                                .clone()
                                .unwrap_or_default()
                                .display_name
                                .clone(),
                            email: reply_0
                                .config_set
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
        let outgoing_2 = sellers::BatchCreateSellersRequest {
            context: req.context.clone(),
            chunk: Some(sellers::batch_create_sellers_request::Chunk::Record(sellers::Seller {
                contact: Some(common::ContactInfo {
                    email: reply_0
                        .config_set
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
            })),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = SellerServiceClient::new(channel);
            let input: tonic::Request<sellers::BatchCreateSellersRequest> =
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
            let response = client.batch_create_sellers(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = featureflags::StreamFlagRolloutChangesRequest {
            frame: Some(featureflags::stream_flag_rollout_changes_request::Frame::Change(
                featureflags::FlagRollout {
                    location: Some(common::PostalAddress {
                        region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                            Some(configuration::environment::Locator::Address(value)) => value,
                            _ => Default::default(),
                        })
                        .region
                        .clone(),
                        ..Default::default()
                    }),
                    organizer: Some(identity::UserRef {
                        email: req
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .owner
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
            let mut client = FlagRolloutSyncServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutChangesRequest> =
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
        let mut response = configuration::SyncEnvironmentsResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_2.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            req.environment.clone().unwrap_or_default().specs.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run PushEnvironment after validating the tenant and durably recording the command.
    async fn run_push_environment(
        &self,
        req: configuration::PushEnvironmentRequest,
    ) -> Result<configuration::PushEnvironmentResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PushEnvironment", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PushEnvironment", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CountConfigChangesRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_changes_request::Filter {
                created_after: req
                    .environment
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: req.annotations.clone(),
                region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::CreateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        manager: Some(identity::UserRef {
                            user_id: req
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeQueryServiceClient::new(channel_0);
                let input: tonic::Request<configuration::CountConfigChangesRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_config_changes(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ConfigSetServiceClient::new(channel_1);
                let input: tonic::Request<configuration::CreateConfigSetRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.create_config_set(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = configuration::PushEnvironmentResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: Vec<String> = reply_0.result.clone().unwrap_or_default().reference_ids.clone();
        if contribution != <Vec<String>>::default() {
            response
                .result
                .get_or_insert_with(Default::default)
                .reference_ids
                .extend(contribution);
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
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i64 = req.environment.clone().unwrap_or_default().capacity.clone();
        if contribution != <i64>::default() {
            response.environment.get_or_insert_with(Default::default).capacity = contribution;
        }
        Ok(response)
    }
    /// Run PullEnvironments after validating the tenant and durably recording the command.
    async fn run_pull_environments(
        &self,
        req: configuration::PullEnvironmentsRequest,
    ) -> Result<configuration::PullEnvironmentsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("PullEnvironments", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "PullEnvironments", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::CreateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_at: req
                                .environment
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_at
                                .clone(),
                            etag: req
                                .environment
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
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::CreateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.create_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ValidateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        location: Some(common::PostalAddress {
                            lines: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .lines
                            .clone(),
                            locality: reply_0
                                .config_set
                                .clone()
                                .unwrap_or_default()
                                .location
                                .clone()
                                .unwrap_or_default()
                                .locality
                                .clone(),
                            recipient: reply_0
                                .config_set
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
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = ConfigSetValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ValidateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.validate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::PullEnvironmentsResponse::default();
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
            .created_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .created_at = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i64 = req.environment.clone().unwrap_or_default().capacity.clone();
        if contribution != <i64>::default() {
            response.environment.get_or_insert_with(Default::default).capacity = contribution;
        }
        Ok(response)
    }
    /// Run ReconcileEnvironments after validating the tenant and durably recording the command.
    async fn run_reconcile_environments(
        &self,
        req: configuration::ReconcileEnvironmentsRequest,
    ) -> Result<configuration::ReconcileEnvironmentsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository.record("ReconcileEnvironments", &context, &req).await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ReconcileEnvironments", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::SimulateConfigSetRequest {
            config_set: Some(configuration::ConfigSet {
                children: vec![configuration::ConfigSet {
                    children: vec![configuration::ConfigSet {
                        audit: Some(common::AuditInfo {
                            created_by: req
                                .environment
                                .clone()
                                .unwrap_or_default()
                                .audit
                                .clone()
                                .unwrap_or_default()
                                .created_by
                                .clone(),
                            etag: req
                                .environment
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
                            locality: (match req.environment.clone().unwrap_or_default().locator.clone() {
                                Some(configuration::environment::Locator::Address(value)) => value,
                                _ => Default::default(),
                            })
                            .locality
                            .clone(),
                            ..Default::default()
                        }),
                        manager: Some(identity::UserRef {
                            display_name: req
                                .environment
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
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = ConfigSetCalculationServiceClient::new(channel);
            let input: tonic::Request<configuration::SimulateConfigSetRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.simulate_config_set(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = featureflags::StreamFlagRolloutExportRequest {
            context: req.context.clone(),
            filter: Some(featureflags::stream_flag_rollout_export_request::Filter {
                created_after: reply_0
                    .config_set
                    .clone()
                    .unwrap_or_default()
                    .audit
                    .clone()
                    .unwrap_or_default()
                    .updated_at
                    .clone(),
                labels: reply_0.result.clone().unwrap_or_default().details.clone(),
                owner_id: reply_0.result.clone().unwrap_or_default().correlation_id.clone(),
                region: reply_0
                    .config_set
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
            let mut client = FlagRolloutExportServiceClient::new(channel);
            let input: tonic::Request<featureflags::StreamFlagRolloutExportRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.stream_flag_rollout_export(input).await?;
            let mut frames = response.into_inner();
            let mut last = featureflags::StreamFlagRolloutExportResponse::default();
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
        let outgoing_2 = configuration::BatchGetConfigSetsRequest {
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_2 = async move {
            let mut client = ConfigSetServiceClient::new(channel);
            let input: tonic::Request<configuration::BatchGetConfigSetsRequest> =
                tonic::IntoRequest::into_request(outgoing_2);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.batch_get_config_sets(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_3 = authorization::ScoreRoleBindingRequest {
            context: req.context.clone(),
            details: Some(authorization::score_role_binding_request::Details {
                destination: Some(common::PostalAddress {
                    postal_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .postal_code
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            role_binding: Some(authorization::RoleBinding {
                audit: Some(common::AuditInfo {
                    created_at: reply_0
                        .config_set
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_at
                        .clone(),
                    ..Default::default()
                }),
                billing_address: Some(common::PostalAddress {
                    region: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .region
                    .clone(),
                    ..Default::default()
                }),
                lines: vec![authorization::RoleBindingLine {
                    description: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .components
                        .clone()
                        .get("primary")
                        .cloned()
                        .unwrap_or_default()
                        .description
                        .clone(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_3 = async move {
            let mut client = RoleBindingValidationServiceClient::new(channel);
            let input: tonic::Request<authorization::ScoreRoleBindingRequest> =
                tonic::IntoRequest::into_request(outgoing_3);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.score_role_binding(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ReconcileEnvironmentsResponse::default();
        let contribution: String = reply_0.config_set.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: String = reply_0
            .config_set
            .clone()
            .unwrap_or_default()
            .manager
            .clone()
            .unwrap_or_default()
            .display_name
            .clone();
        if contribution != <String>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
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
        let contribution: String = reply_2.result.clone().unwrap_or_default().message.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).message = contribution;
        }
        let contribution: String = reply_3.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: Option<prost_types::Timestamp> = reply_3
            .role_binding
            .clone()
            .unwrap_or_default()
            .audit
            .clone()
            .unwrap_or_default()
            .updated_at
            .clone();
        if contribution != <Option<prost_types::Timestamp>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .audit
                .get_or_insert_with(Default::default)
                .updated_at = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: i64 = req.environment.clone().unwrap_or_default().capacity.clone();
        if contribution != <i64>::default() {
            response.environment.get_or_insert_with(Default::default).capacity = contribution;
        }
        Ok(response)
    }
    /// Run StreamEnvironmentChanges after validating the tenant and durably recording the command.
    async fn run_stream_environment_changes(
        &self,
        req: configuration::StreamEnvironmentChangesRequest,
    ) -> Result<configuration::StreamEnvironmentChangesResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("StreamEnvironmentChanges", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "StreamEnvironmentChanges", "running workflow");
        tracing::debug!(audit_value = ?(match req.frame.clone() { Some(configuration::stream_environment_changes_request::Frame::Change(value)) => value, _ => Default::default() }).environment_id.clone(), "recorded audit field");
        let outgoing_0 = configuration::ExplainConfigChangeCalculationRequest {
            context: req.context.clone(),
            details: Some(configuration::explain_config_change_calculation_request::Details {
                contact: Some(common::ContactInfo {
                    email: (match req.frame.clone() {
                        Some(configuration::stream_environment_changes_request::Frame::Change(value)) => value,
                        _ => Default::default(),
                    })
                    .owner
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
        let channel_0 = self.channel.clone();
        let outgoing_1 = configuration::CountConfigSetTotalsRequest {
            context: req.context.clone(),
            filter: Some(configuration::count_config_set_totals_request::Filter {
                created_after: (match req.frame.clone() {
                    Some(configuration::stream_environment_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .audit
                .clone()
                .unwrap_or_default()
                .updated_at
                .clone(),
                region: (match (match req.frame.clone() {
                    Some(configuration::stream_environment_changes_request::Frame::Change(value)) => value,
                    _ => Default::default(),
                })
                .locator
                .clone()
                {
                    Some(configuration::environment::Locator::Address(value)) => value,
                    _ => Default::default(),
                })
                .region
                .clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel_1 = self.channel.clone();
        let (reply_0, reply_1) = tokio::join!(
            async move {
                let mut client = ConfigChangeCalculationServiceClient::new(channel_0);
                let input: tonic::Request<configuration::ExplainConfigChangeCalculationRequest> =
                    tonic::IntoRequest::into_request(outgoing_0);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.explain_config_change_calculation(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            },
            async move {
                let mut client = ConfigSetReportingServiceClient::new(channel_1);
                let input: tonic::Request<configuration::CountConfigSetTotalsRequest> =
                    tonic::IntoRequest::into_request(outgoing_1);
                let context = input.get_ref().context.clone().unwrap_or_default();
                let input = acme_platform::auth::service_request(input, &context)?;
                let response = client.count_config_set_totals(input).await?;
                Ok::<_, tonic::Status>(response.into_inner())
            }
        );
        let reply_0 = reply_0?;
        let reply_1 = reply_1?;
        let mut response = configuration::StreamEnvironmentChangesResponse::default();
        let contribution: i64 = reply_0.config_change.clone().unwrap_or_default().sequence.clone();
        if contribution != <i64>::default() {
            response.sequence = contribution;
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
        let contribution: String = reply_1.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = (match req.frame.clone() {
            Some(configuration::stream_environment_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .name
        .clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> = (match req.frame.clone() {
            Some(configuration::stream_environment_changes_request::Frame::Change(value)) => value,
            _ => Default::default(),
        })
        .specs
        .clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
                .extend(contribution);
        }
        Ok(response)
    }
    /// Run ResolveEnvironmentConflict after validating the tenant and durably recording the command.
    async fn run_resolve_environment_conflict(
        &self,
        req: configuration::ResolveEnvironmentConflictRequest,
    ) -> Result<configuration::ResolveEnvironmentConflictResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ResolveEnvironmentConflict", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ResolveEnvironmentConflict", "running workflow");
        tracing::debug!(audit_value = ?req.environment.clone().unwrap_or_default().environment_id.clone(), "recorded audit field");
        let outgoing_0 = ratelimiting::QuoteQuotaBucketRequest {
            context: req.context.clone(),
            details: Some(ratelimiting::quote_quota_bucket_request::Details {
                destination: Some(common::PostalAddress {
                    lines: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .lines
                    .clone(),
                    postal_code: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .postal_code
                    .clone(),
                    recipient: (match req.environment.clone().unwrap_or_default().locator.clone() {
                        Some(configuration::environment::Locator::Address(value)) => value,
                        _ => Default::default(),
                    })
                    .recipient
                    .clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = QuotaBucketCalculationServiceClient::new(channel);
            let input: tonic::Request<ratelimiting::QuoteQuotaBucketRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_quota_bucket(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let outgoing_1 = configuration::ExplainEnvironmentDecisionRequest {
            context: req.context.clone(),
            environment: Some(configuration::Environment {
                audit: Some(common::AuditInfo {
                    created_by: req
                        .environment
                        .clone()
                        .unwrap_or_default()
                        .audit
                        .clone()
                        .unwrap_or_default()
                        .created_by
                        .clone(),
                    ..Default::default()
                }),
                components: std::collections::BTreeMap::from([(
                    "primary".to_owned(),
                    configuration::EnvironmentComponent {
                        description: req
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .components
                            .clone()
                            .get("primary")
                            .cloned()
                            .unwrap_or_default()
                            .description
                            .clone(),
                        quantity: req
                            .environment
                            .clone()
                            .unwrap_or_default()
                            .components
                            .clone()
                            .get("primary")
                            .cloned()
                            .unwrap_or_default()
                            .quantity
                            .clone(),
                        ..Default::default()
                    },
                )]),
                owner: Some(identity::UserRef {
                    display_name: reply_0
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
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_1 = async move {
            let mut client = EnvironmentValidationServiceClient::new(channel);
            let input: tonic::Request<configuration::ExplainEnvironmentDecisionRequest> =
                tonic::IntoRequest::into_request(outgoing_1);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.explain_environment_decision(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::ResolveEnvironmentConflictResponse::default();
        let contribution: String = reply_0.result.clone().unwrap_or_default().correlation_id.clone();
        if contribution != <String>::default() {
            response.result.get_or_insert_with(Default::default).correlation_id = contribution;
        }
        let contribution: String = reply_0.result.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: String = reply_1.outcome.clone().unwrap_or_default().status.clone();
        if contribution != <String>::default() {
            response.outcome.get_or_insert_with(Default::default).status = contribution;
        }
        let contribution: std::collections::BTreeMap<String, String> =
            reply_1.environment.clone().unwrap_or_default().specs.clone();
        if contribution != <std::collections::BTreeMap<String, String>>::default() {
            response
                .environment
                .get_or_insert_with(Default::default)
                .specs
                .extend(contribution);
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        let contribution: String = req.environment.clone().unwrap_or_default().name.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).name = contribution;
        }
        Ok(response)
    }
    /// Run GetEnvironmentSyncState after validating the tenant and durably recording the command.
    async fn run_get_environment_sync_state(
        &self,
        req: configuration::GetEnvironmentSyncStateRequest,
    ) -> Result<configuration::GetEnvironmentSyncStateResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("GetEnvironmentSyncState", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "GetEnvironmentSyncState", "running workflow");
        let outgoing_0 = certificates::QuoteCertificateRequest {
            certificate: Some(certificates::Certificate {
                audit: Some(common::AuditInfo {
                    updated_at: req.context.clone().unwrap_or_default().deadline.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            context: req.context.clone(),
            ..Default::default()
        };
        let channel = self.channel.clone();
        let reply_0 = async move {
            let mut client = CertificateCalculationServiceClient::new(channel);
            let input: tonic::Request<certificates::QuoteCertificateRequest> =
                tonic::IntoRequest::into_request(outgoing_0);
            let context = input.get_ref().context.clone().unwrap_or_default();
            let input = acme_platform::auth::service_request(input, &context)?;
            let response = client.quote_certificate(input).await?;
            Ok::<_, tonic::Status>(response.into_inner())
        }
        .await?;
        let mut response = configuration::GetEnvironmentSyncStateResponse::default();
        let contribution: String = reply_0
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
                .environment
                .get_or_insert_with(Default::default)
                .owner
                .get_or_insert_with(Default::default)
                .display_name = contribution;
        }
        let contribution: String = req.environment_id.clone();
        if contribution != <String>::default() {
            response.environment.get_or_insert_with(Default::default).environment_id = contribution;
        }
        Ok(response)
    }
    /// Run ListEnvironmentConflicts after validating the tenant and durably recording the command.
    async fn run_list_environment_conflicts(
        &self,
        req: configuration::ListEnvironmentConflictsRequest,
    ) -> Result<configuration::ListEnvironmentConflictsResponse, tonic::Status> {
        acme_platform::validate(req.context.as_ref())?;
        let context = req.context.clone().unwrap_or_default();
        self.repository
            .record("ListEnvironmentConflicts", &context, &req)
            .await?;
        tracing::info!(tenant = %context.tenant_id, operation = "ListEnvironmentConflicts", "running workflow");
        tracing::debug!(audit_value = ?req.page.clone().unwrap_or_default().page_token.clone(), "recorded audit field");
        let response = configuration::ListEnvironmentConflictsResponse::default();
        Ok(response)
    }
}
acme_platform::endpoints! { EnvironmentSyncService, EnvironmentSyncEndpoint;
unary {
sync_environments (configuration::SyncEnvironmentsRequest) -> configuration::SyncEnvironmentsResponse => run_sync_environments;
push_environment (configuration::PushEnvironmentRequest) -> configuration::PushEnvironmentResponse => run_push_environment;
pull_environments (configuration::PullEnvironmentsRequest) -> configuration::PullEnvironmentsResponse => run_pull_environments;
reconcile_environments (configuration::ReconcileEnvironmentsRequest) -> configuration::ReconcileEnvironmentsResponse => run_reconcile_environments;
resolve_environment_conflict (configuration::ResolveEnvironmentConflictRequest) -> configuration::ResolveEnvironmentConflictResponse => run_resolve_environment_conflict;
get_environment_sync_state (configuration::GetEnvironmentSyncStateRequest) -> configuration::GetEnvironmentSyncStateResponse => run_get_environment_sync_state;
list_environment_conflicts (configuration::ListEnvironmentConflictsRequest) -> configuration::ListEnvironmentConflictsResponse => run_list_environment_conflicts;
}
server {
}
client {
}
bidi {
stream_environment_changes (configuration::StreamEnvironmentChangesRequest) -> configuration::StreamEnvironmentChangesResponse, StreamEnvironmentChangesStream => run_stream_environment_changes;
}
}
