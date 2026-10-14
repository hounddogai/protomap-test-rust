//! `settle_secret_workflow_message` reads `acme.secrets.v1.SecretWorkflowService/SignalSecretWorkflow` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = secrets::secret_workflow_service_client::SecretWorkflowServiceClient::new(channel.clone());
    let found = source
        .signal_secret_workflow(secrets::SignalSecretWorkflowRequest {
            context: Some(context.clone()),
            secret_id: "current-settle-secret-workflow-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let secret = found.secret.unwrap_or_default();
    let record = SecretWorkflowMessageRecord {
        message: outcome.message,
        result_message: result.message,
        currency_code: secret.currency_code,
        secret_id: secret.secret_id,
    };
    dispatch_secret_workflow_message("video-visit-calculation", record, channel, &context).await
}

/// What `settle_secret_workflow_message` keeps of each SignalSecretWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct SecretWorkflowMessageRecord {
    pub message: String,
    pub result_message: String,
    pub currency_code: String,
    pub secret_id: String,
}

/// Where `settle_secret_workflow_message` hands each record.
trait SecretWorkflowMessageDestination {
    fn push(&self, record: SecretWorkflowMessageRecord) -> Delivery;
}

/// Writes records to `acme.telehealth.v1.VideoVisitCalculationService`.
struct VideoVisitCalculationSink {
    client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient<Channel>,
}

impl SecretWorkflowMessageDestination for VideoVisitCalculationSink {
    fn push(&self, record: SecretWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_video_visits(telehealth::RecalculateVideoVisitsRequest {
                    details: Some(telehealth::recalculate_video_visits_request::Details {
                        currency_code: record.result_message,
                        ..Default::default()
                    }),
                    video_visit: Some(telehealth::VideoVisit {
                        title: record.secret_id,
                        ..Default::default()
                    }),
                    video_visit_id: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.AccessPolicyNotificationService`.
struct AccessPolicyNotificationSink {
    client: authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient<Channel>,
}

impl SecretWorkflowMessageDestination for AccessPolicyNotificationSink {
    fn push(&self, record: SecretWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_access_policy_subscriptions(authorization::ListAccessPolicySubscriptionsRequest {
                    filter: Some(authorization::list_access_policy_subscriptions_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    order_by: record.result_message,
                    page: Some(common::PageRequest {
                        page_token: record.secret_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.edgegateway.v1.ApiKeyService`.
struct ApiKeySink {
    client: edgegateway::api_key_service_client::ApiKeyServiceClient<Channel>,
}

impl SecretWorkflowMessageDestination for ApiKeySink {
    fn push(&self, record: SecretWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_api_key(edgegateway::GetApiKeyRequest {
                    api_key_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CertificateAuthorityQueryService`.
struct CertificateAuthorityQuerySink {
    client: certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient<Channel>,
}

impl SecretWorkflowMessageDestination for CertificateAuthorityQuerySink {
    fn push(&self, record: SecretWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_certificate_authorities(certificates::FindCertificateAuthoritiesRequest {
                    filter: Some(certificates::find_certificate_authorities_request::Filter {
                        owner_id: record.secret_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct SecretWorkflowMessageLogOnly;

impl SecretWorkflowMessageDestination for SecretWorkflowMessageLogOnly {
    fn push(&self, record: SecretWorkflowMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_secret_workflow_message(
    kind: &str,
    record: SecretWorkflowMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let video_visit_calculation = VideoVisitCalculationSink {
        client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let access_policy_notification = AccessPolicyNotificationSink {
        client: authorization::access_policy_notification_service_client::AccessPolicyNotificationServiceClient::new(
            channel.clone(),
        ),
    };
    let api_key = ApiKeySink {
        client: edgegateway::api_key_service_client::ApiKeyServiceClient::new(channel.clone()),
    };
    let certificate_authority_query = CertificateAuthorityQuerySink {
        client: certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient::new(
            channel,
        ),
    };
    let write: &dyn Fn(SecretWorkflowMessageRecord) -> Delivery = match kind {
        "video-visit-calculation" => &|record| video_visit_calculation.push(record),
        "access-policy-notification" => &|record| access_policy_notification.push(record),
        "api-key" => &|record| api_key.push(record),
        "certificate-authority-query" => &|record| certificate_authority_query.push(record),
        _ => &|record| SecretWorkflowMessageLogOnly.push(record),
    };
    write(record).await
}
