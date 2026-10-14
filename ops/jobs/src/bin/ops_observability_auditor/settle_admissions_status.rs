//! `settle_admissions_status` reads `acme.admissions.v1.AdmissionQueryService/ResolveAdmissions` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::forms::v1 as forms;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = admissions::admission_query_service_client::AdmissionQueryServiceClient::new(channel.clone());
    let fetched = source
        .resolve_admissions(admissions::ResolveAdmissionsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let page_info = fetched.page_info.unwrap_or_default();
    let record = AdmissionsStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
    };
    deliver_admissions_status("crypto-key-calculation", record, channel, &context).await
}

/// What `settle_admissions_status` keeps of each ResolveAdmissions response.
#[derive(Clone, Debug, Default)]
pub struct AdmissionsStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub next_page_token: String,
}

/// Where `settle_admissions_status` hands each record.
trait AdmissionsStatusTarget {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery;
}

/// Writes records to `acme.forms.v1.FormCalculationService`.
struct FormCalculationSink {
    client: forms::form_calculation_service_client::FormCalculationServiceClient<Channel>,
}

impl AdmissionsStatusTarget for FormCalculationSink {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_form(forms::SimulateFormRequest {
                    form_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl AdmissionsStatusTarget for CryptoKeyCalculationSink {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::RecalculateCryptoKeysRequest {
                    reason: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.recalculate_crypto_keys(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.RateLimitService`.
struct RateLimitSink {
    client: ratelimiting::rate_limit_service_client::RateLimitServiceClient<Channel>,
}

impl AdmissionsStatusTarget for RateLimitSink {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_rate_limits(ratelimiting::ListRateLimitsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.next_page_token,
                        ..Default::default()
                    }),
                    filter: Some(ratelimiting::list_rate_limits_request::Filter {
                        region: record.correlation_id,
                        owner_id: record.status,
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
struct AdmissionsStatusLogOnly;

impl AdmissionsStatusTarget for AdmissionsStatusLogOnly {
    fn deliver(&self, record: AdmissionsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_admissions_status(
    kind: &str,
    record: AdmissionsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AdmissionsStatusTarget> = match kind {
        "form-calculation" => Box::new(FormCalculationSink {
            client: forms::form_calculation_service_client::FormCalculationServiceClient::new(channel.clone()),
        }),
        "crypto-key-calculation" => Box::new(CryptoKeyCalculationSink {
            client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "rate-limit" => Box::new(RateLimitSink {
            client: ratelimiting::rate_limit_service_client::RateLimitServiceClient::new(channel),
        }),
        _ => Box::new(AdmissionsStatusLogOnly),
    };
    sink.deliver(record).await
}
