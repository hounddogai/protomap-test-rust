//! `backfill_admission_code` reads `acme.admissions.v1.AdmissionCalculationService/SimulateAdmission` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::loyalty::v1 as loyalty;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(channel.clone());
    let found = source
        .simulate_admission(admissions::SimulateAdmissionRequest {
            context: Some(context.clone()),
            admission_id: "live-backfill-admission-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = found.breakdown.unwrap_or_default();
    let admission = found.admission.unwrap_or_default();
    let record = AdmissionCodeRecord {
        currency_code: breakdown.currency_code,
        admission_id: admission.admission_id,
        source: admission.source,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AdmissionCodeSink>> = BTreeMap::new();
    sinks.insert(
        "access-policy-history",
        Box::new(AccessPolicyHistorySink {
            client: authorization::access_policy_history_service_client::AccessPolicyHistoryServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "loyalty-member-query",
        Box::new(LoyaltyMemberQuerySink {
            client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let Some(sink) = sinks.get("access-policy-history") else {
        anyhow::bail!("no sink registered for access-policy-history");
    };
    sink.apply(record).await
}

/// What `backfill_admission_code` keeps of each SimulateAdmission response.
#[derive(Clone, Debug, Default)]
pub struct AdmissionCodeRecord {
    pub currency_code: String,
    pub admission_id: String,
    pub source: String,
}

/// Where `backfill_admission_code` hands each record.
trait AdmissionCodeSink {
    fn apply(&self, record: AdmissionCodeRecord) -> Delivery;
}

/// Passes records to `acme.authorization.v1.AccessPolicyHistoryService`.
struct AccessPolicyHistorySink {
    client: authorization::access_policy_history_service_client::AccessPolicyHistoryServiceClient<Channel>,
}

impl AdmissionCodeSink for AccessPolicyHistorySink {
    fn apply(&self, record: AdmissionCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_access_policy_versions(authorization::ListAccessPolicyVersionsRequest {
                    filter: Some(authorization::list_access_policy_versions_request::Filter {
                        owner_id: record.admission_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.loyalty.v1.LoyaltyMemberQueryService`.
struct LoyaltyMemberQuerySink {
    client: loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl AdmissionCodeSink for LoyaltyMemberQuerySink {
    fn apply(&self, record: AdmissionCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(loyalty::LookupLoyaltyMemberRequest {
                    loyalty_member_id: record.source,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_loyalty_member(request).await?;
            Ok(())
        })
    }
}
