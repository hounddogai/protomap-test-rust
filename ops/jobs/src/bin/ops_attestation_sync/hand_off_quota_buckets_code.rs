//! `hand_off_quota_buckets_code` reads
//! `acme.ratelimiting.v1.QuotaBucketCalculationService/RecalculateQuotaBuckets` and hands what it keeps to
//! `acme.admissions.v1.AdmissionQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient::new(
        channel.clone(),
    );
    let fetched = source
        .recalculate_quota_buckets(ratelimiting::RecalculateQuotaBucketsRequest {
            context: Some(context.clone()),
            reason: "current-hand-off-quota-buckets-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = fetched.breakdown.unwrap_or_default();
    let quota_bucket = fetched.quota_bucket.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = OffQuotaBucketsCodeRecord {
        currency_code: breakdown.currency_code,
        quota_bucket_id: acme_kit::text::normalize(&quota_bucket.quota_bucket_id),
        message: acme_kit::text::normalize(&result.message),
    };
    AdmissionQuerySink {
        client: admissions::admission_query_service_client::AdmissionQueryServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `hand_off_quota_buckets_code` keeps of each RecalculateQuotaBuckets response.
#[derive(Clone, Debug, Default)]
pub struct OffQuotaBucketsCodeRecord {
    pub currency_code: String,
    pub quota_bucket_id: String,
    pub message: String,
}

/// Where `hand_off_quota_buckets_code` hands each record.
trait OffQuotaBucketsCodeWriter {
    fn forward(&self, record: OffQuotaBucketsCodeRecord) -> Delivery;
}

/// Writes records to `acme.admissions.v1.AdmissionQueryService`.
struct AdmissionQuerySink {
    client: admissions::admission_query_service_client::AdmissionQueryServiceClient<Channel>,
}

impl OffQuotaBucketsCodeWriter for AdmissionQuerySink {
    fn forward(&self, record: OffQuotaBucketsCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_admissions(admissions::BrowseAdmissionsRequest {
                    order_by: record.currency_code,
                    filter: Some(admissions::browse_admissions_request::Filter {
                        region: record.quota_bucket_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
