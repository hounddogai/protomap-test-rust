//! `mirror_secret_id` reads `acme.secrets.v1.SecretService/GetSecret` and hands what it keeps to the sink of
//! its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = secrets::secret_service_client::SecretServiceClient::new(channel.clone());
    let fetched = source
        .get_secret(secrets::GetSecretRequest {
            context: Some(context.clone()),
            secret_id: "live-mirror-secret-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let secret = fetched.secret.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = SecretIdRecord {
        secret_id: secret.secret_id,
        status: acme_kit::text::normalize(&result.status),
        message: acme_kit::text::normalize(&result.message),
        currency_code: secret.currency_code,
    };
    let mut sinks: BTreeMap<&str, Box<dyn SecretIdTarget>> = BTreeMap::new();
    sinks.insert(
        "user-validation",
        Box::new(UserValidationSink {
            client: identity::user_validation_service_client::UserValidationServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "key-ring-reporting",
        Box::new(KeyRingReportingSink {
            client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "permission-check-export",
        Box::new(PermissionCheckExportSink {
            client: authorization::permission_check_export_service_client::PermissionCheckExportServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "role-binding-admin",
        Box::new(RoleBindingAdminSink {
            client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("user-validation") else {
        anyhow::bail!("no sink registered for user-validation");
    };
    sink.apply(record).await
}

/// What `mirror_secret_id` keeps of each GetSecret response.
#[derive(Clone, Debug, Default)]
pub struct SecretIdRecord {
    pub secret_id: String,
    pub status: String,
    pub message: String,
    pub currency_code: String,
}

/// Where `mirror_secret_id` hands each record.
trait SecretIdTarget {
    fn apply(&self, record: SecretIdRecord) -> Delivery;
}

/// Passes records to `acme.identity.v1.UserValidationService`.
struct UserValidationSink {
    client: identity::user_validation_service_client::UserValidationServiceClient<Channel>,
}

impl SecretIdTarget for UserValidationSink {
    fn apply(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_user_decision(identity::ExplainUserDecisionRequest {
                    user_id: record.message,
                    user: Some(identity::User {
                        user_id: record.secret_id,
                        ..Default::default()
                    }),
                    reason: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRingReportingService`.
struct KeyRingReportingSink {
    client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient<Channel>,
}

impl SecretIdTarget for KeyRingReportingSink {
    fn apply(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .schedule_key_ring_report(keymanagement::ScheduleKeyRingReportRequest {
                    details: Some(keymanagement::schedule_key_ring_report_request::Details {
                        cron: record.message,
                        format: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.PermissionCheckExportService`.
struct PermissionCheckExportSink {
    client: authorization::permission_check_export_service_client::PermissionCheckExportServiceClient<Channel>,
}

impl SecretIdTarget for PermissionCheckExportSink {
    fn apply(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .schedule_permission_check_export(authorization::SchedulePermissionCheckExportRequest {
                    permission_check_id: record.currency_code,
                    details: Some(authorization::schedule_permission_check_export_request::Details {
                        cron: record.status,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.authorization.v1.RoleBindingAdminService`.
struct RoleBindingAdminSink {
    client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient<Channel>,
}

impl SecretIdTarget for RoleBindingAdminSink {
    fn apply(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lock_role_binding(authorization::LockRoleBindingRequest {
                    details: Some(authorization::lock_role_binding_request::Details {
                        reason_code: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
