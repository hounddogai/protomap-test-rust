//! `backfill_conversation_options_status` reads
//! `acme.messaging.v1.ConversationCalculationService/CompareConversationOptions` and hands what it keeps to
//! `acme.anomalies.v1.AnomalyQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::messaging::v1 as messaging;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        messaging::conversation_calculation_service_client::ConversationCalculationServiceClient::new(channel.clone());
    let fetched = source
        .compare_conversation_options(messaging::CompareConversationOptionsRequest {
            context: Some(context.clone()),
            reason: "primary-backfill-conversation-options-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let breakdown = fetched.breakdown.unwrap_or_default();
    let record = ConversationOptionsStatusRecord {
        status: result.status,
        currency_code: breakdown.currency_code,
    };
    AnomalyQuerySink {
        client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel),
        context: context.clone(),
    }
    .apply(record)
    .await
}

/// What `backfill_conversation_options_status` keeps of each CompareConversationOptions response.
#[derive(Clone, Debug, Default)]
pub struct ConversationOptionsStatusRecord {
    pub status: String,
    pub currency_code: String,
}

/// Where `backfill_conversation_options_status` hands each record.
trait ConversationOptionsStatusWriter {
    fn apply(&self, record: ConversationOptionsStatusRecord) -> Delivery;
}

/// Sends records to `acme.anomalies.v1.AnomalyQueryService`.
struct AnomalyQuerySink {
    client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConversationOptionsStatusWriter for AnomalyQuerySink {
    fn apply(&self, record: ConversationOptionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(anomalies::SummarizeAnomaliesRequest {
                    details: Some(anomalies::summarize_anomalies_request::Details {
                        cron: record.currency_code,
                        ..Default::default()
                    }),
                    anomaly_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_anomalies(request).await?;
            Ok(())
        })
    }
}
