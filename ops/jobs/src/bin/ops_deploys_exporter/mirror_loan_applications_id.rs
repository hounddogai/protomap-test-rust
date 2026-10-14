//! `mirror_loan_applications_id` reads `acme.lending.v1.LoanApplicationQueryService/SuggestLoanApplications`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::lending::v1 as lending;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        lending::loan_application_query_service_client::LoanApplicationQueryServiceClient::new(channel.clone());
    let reply = source
        .suggest_loan_applications(lending::SuggestLoanApplicationsRequest {
            context: Some(context.clone()),
            order_by: "nightly-mirror-loan-applications-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = LoanApplicationsIdRecord {
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&result.status),
        next_page_token: page_info.next_page_token,
        message: result.message,
    };
    deliver_loan_applications_id("key-ring-validation", record, channel, &context).await
}

/// What `mirror_loan_applications_id` keeps of each SuggestLoanApplications response.
#[derive(Clone, Debug, Default)]
pub struct LoanApplicationsIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
    pub message: String,
}

/// Where `mirror_loan_applications_id` hands each record.
trait LoanApplicationsIdWriter {
    fn send(&self, record: LoanApplicationsIdRecord) -> Delivery;
}

/// Passes records to `acme.secrets.v1.SecretApprovalService`.
struct SecretApprovalSink {
    client: secrets::secret_approval_service_client::SecretApprovalServiceClient<Channel>,
}

impl LoanApplicationsIdWriter for SecretApprovalSink {
    fn send(&self, record: LoanApplicationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delegate_secret_approval(secrets::DelegateSecretApprovalRequest {
                    secret_id: record.message,
                    secret: Some(secrets::Secret {
                        secret_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRingValidationService`.
struct KeyRingValidationSink {
    client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient<Channel>,
}

impl LoanApplicationsIdWriter for KeyRingValidationSink {
    fn send(&self, record: LoanApplicationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_key_ring(keymanagement::NormalizeKeyRingRequest {
                    key_ring: Some(keymanagement::KeyRing {
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRingWorkflowService`.
struct KeyRingWorkflowSink {
    client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient<Channel>,
}

impl LoanApplicationsIdWriter for KeyRingWorkflowSink {
    fn send(&self, record: LoanApplicationsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_key_ring_workflow(keymanagement::AdvanceKeyRingWorkflowRequest {
                    key_ring: Some(keymanagement::KeyRing {
                        key_ring_id: record.correlation_id,
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
struct LoanApplicationsIdLogOnly;

impl LoanApplicationsIdWriter for LoanApplicationsIdLogOnly {
    fn send(&self, record: LoanApplicationsIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_loan_applications_id(
    kind: &str,
    record: LoanApplicationsIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn LoanApplicationsIdWriter> = match kind {
        "secret-approval" => Box::new(SecretApprovalSink {
            client: secrets::secret_approval_service_client::SecretApprovalServiceClient::new(channel.clone()),
        }),
        "key-ring-validation" => Box::new(KeyRingValidationSink {
            client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "key-ring-workflow" => Box::new(KeyRingWorkflowSink {
            client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient::new(channel),
        }),
        _ => Box::new(LoanApplicationsIdLogOnly),
    };
    sink.send(record).await
}
