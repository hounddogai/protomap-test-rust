//! `carry_ledger_accounts_token` reads `acme.ledger.v1.LedgerAccountQueryService/ResolveLedgerAccounts` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::messaging::v1 as messaging;
use acme_proto::acme::patients::v1 as patients;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient::new(channel.clone());
    let reply = source
        .resolve_ledger_accounts(ledger::ResolveLedgerAccountsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = reply.page_info.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = LedgerAccountsTokenRecord {
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn LedgerAccountsTokenTarget>> = BTreeMap::new();
    sinks.insert(
        "patient-calculation",
        Box::new(PatientCalculationSink {
            client: patients::patient_calculation_service_client::PatientCalculationServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "conversation-calculation",
        Box::new(ConversationCalculationSink {
            client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "environment-workflow",
        Box::new(EnvironmentWorkflowSink {
            client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let Some(sink) = sinks.get("environment-workflow") else {
        anyhow::bail!("no sink registered for environment-workflow");
    };
    sink.deliver(record).await
}

/// What `carry_ledger_accounts_token` keeps of each ResolveLedgerAccounts response.
#[derive(Clone, Debug, Default)]
pub struct LedgerAccountsTokenRecord {
    pub next_page_token: String,
    pub status: String,
}

/// Where `carry_ledger_accounts_token` hands each record.
trait LedgerAccountsTokenTarget {
    fn deliver(&self, record: LedgerAccountsTokenRecord) -> Delivery;
}

/// Writes records to `acme.patients.v1.PatientCalculationService`.
struct PatientCalculationSink {
    client: patients::patient_calculation_service_client::PatientCalculationServiceClient<Channel>,
}

impl LedgerAccountsTokenTarget for PatientCalculationSink {
    fn deliver(&self, record: LedgerAccountsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_patient_rates(patients::GetPatientRatesRequest {
                    patient_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.messaging.v1.ConversationCalculationService`.
struct ConversationCalculationSink {
    client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient<Channel>,
}

impl LedgerAccountsTokenTarget for ConversationCalculationSink {
    fn deliver(&self, record: LedgerAccountsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_conversation(messaging::CalculateConversationRequest {
                    conversation: Some(messaging::Conversation {
                        subject: record.status,
                        ..Default::default()
                    }),
                    details: Some(messaging::calculate_conversation_request::Details {
                        currency_code: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl LedgerAccountsTokenTarget for EnvironmentWorkflowSink {
    fn deliver(&self, record: LedgerAccountsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::CancelEnvironmentWorkflowRequest {
                    environment_id: record.status,
                    environment: Some(configuration::Environment {
                        environment_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.cancel_environment_workflow(request).await?;
            Ok(())
        })
    }
}
