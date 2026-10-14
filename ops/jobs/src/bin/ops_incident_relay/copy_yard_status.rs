//! `copy_yard_status` reads `acme.yards.v1.YardService/DeleteYard` and hands what it keeps to the sink of its
//! kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = yards::yard_service_client::YardServiceClient::new(channel.clone());
    let reply = source
        .delete_yard(yards::DeleteYardRequest {
            context: Some(context.clone()),
            reason: "current-copy-yard-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = YardStatusRecord {
        status: result.status,
        message: result.message,
        correlation_id: result.correlation_id,
    };
    act_on_yard_status("flag-evaluation-sync", record, channel, context).await
}

/// What `copy_yard_status` keeps of each DeleteYard response.
#[derive(Clone, Debug, Default)]
pub struct YardStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `copy_yard_status` hands each record.
trait YardStatusDestination {
    fn apply(&self, record: YardStatusRecord) -> Delivery;
}

/// Hands records to `acme.certificates.v1.CertificateAuthorityWorkflowService`.
struct CertificateAuthorityWorkflowSink {
    client:
        certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardStatusDestination for CertificateAuthorityWorkflowSink {
    fn apply(&self, record: YardStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::SignalCertificateAuthorityWorkflowRequest {
                    certificate_authority: Some(certificates::CertificateAuthority {
                        parent_id: record.message,
                        ..Default::default()
                    }),
                    certificate_authority_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.signal_certificate_authority_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationSyncService`.
struct FlagEvaluationSyncSink {
    client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardStatusDestination for FlagEvaluationSyncSink {
    fn apply(&self, record: YardStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::SyncFlagEvaluationsRequest {
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        flag_evaluation_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.sync_flag_evaluations(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.secrets.v1.SecretVersionQueryService`.
struct SecretVersionQuerySink {
    client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient<Channel>,
}

impl YardStatusDestination for SecretVersionQuerySink {
    fn apply(&self, record: YardStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_secret_version(secrets::PeekSecretVersionRequest {
                    secret_version_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_yard_status(
    kind: &str,
    record: YardStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(YardStatusRecord) -> Delivery>> = BTreeMap::new();
    let certificate_authority_workflow_channel = channel.clone();
    let certificate_authority_workflow_context = context.clone();
    actions.insert("certificate-authority-workflow", Box::new(move |record| CertificateAuthorityWorkflowSink { client: certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient::new(certificate_authority_workflow_channel.clone()), context: certificate_authority_workflow_context.clone() }.apply(record)));
    let flag_evaluation_sync_channel = channel.clone();
    let flag_evaluation_sync_context = context.clone();
    actions.insert(
        "flag-evaluation-sync",
        Box::new(move |record| {
            FlagEvaluationSyncSink {
                client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient::new(
                    flag_evaluation_sync_channel.clone(),
                ),
                context: flag_evaluation_sync_context.clone(),
            }
            .apply(record)
        }),
    );
    let secret_version_query_channel = channel.clone();
    actions.insert(
        "secret-version-query",
        Box::new(move |record| {
            SecretVersionQuerySink {
                client: secrets::secret_version_query_service_client::SecretVersionQueryServiceClient::new(
                    secret_version_query_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
