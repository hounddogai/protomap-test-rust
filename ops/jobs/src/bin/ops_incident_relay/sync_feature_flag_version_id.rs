//! `sync_feature_flag_version_id` reads
//! `acme.featureflags.v1.FeatureFlagHistoryService/AnnotateFeatureFlagVersion` and hands what it keeps to
//! `acme.patientapp.v1.PatientHomeQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::patientapp::v1 as patientapp;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(channel.clone());
    let found = source
        .annotate_feature_flag_version(featureflags::AnnotateFeatureFlagVersionRequest {
            context: Some(context.clone()),
            reason: "live-sync-feature-flag-version-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let feature_flag = found.feature_flag.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = FeatureFlagVersionIdRecord {
        feature_flag_id: feature_flag.feature_flag_id,
        message: result.message,
        name: feature_flag.name,
        description: feature_flag.description,
    };
    PatientHomeQuerySink {
        client: patientapp::patient_home_query_service_client::PatientHomeQueryServiceClient::new(channel),
    }
    .apply(record)
    .await
}

/// What `sync_feature_flag_version_id` keeps of each AnnotateFeatureFlagVersion response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagVersionIdRecord {
    pub feature_flag_id: String,
    pub message: String,
    pub name: String,
    pub description: String,
}

/// Where `sync_feature_flag_version_id` hands each record.
trait FeatureFlagVersionIdSink {
    fn apply(&self, record: FeatureFlagVersionIdRecord) -> Delivery;
}

/// Passes records to `acme.patientapp.v1.PatientHomeQueryService`.
struct PatientHomeQuerySink {
    client: patientapp::patient_home_query_service_client::PatientHomeQueryServiceClient<Channel>,
}

impl FeatureFlagVersionIdSink for PatientHomeQuerySink {
    fn apply(&self, record: FeatureFlagVersionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_patient_homes(patientapp::BrowsePatientHomesRequest {
                    order_by: record.description,
                    filter: Some(patientapp::browse_patient_homes_request::Filter {
                        region: record.feature_flag_id,
                        query: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
