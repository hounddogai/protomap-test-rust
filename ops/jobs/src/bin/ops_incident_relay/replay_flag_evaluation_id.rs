//! `replay_flag_evaluation_id` reads `acme.featureflags.v1.FlagEvaluationAdminService/ReassignFlagEvaluation`
//! and hands what it keeps to `acme.certificates.v1.CertificateAuthorityCalculationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(channel.clone());
    let fetched = source
        .reassign_flag_evaluation(featureflags::ReassignFlagEvaluationRequest {
            context: Some(context.clone()),
            reason: "live-replay-flag-evaluation-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_evaluation = fetched.flag_evaluation.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = FlagEvaluationIdRecord {
        flag_evaluation_id: flag_evaluation.flag_evaluation_id,
        message: outcome.message,
        kind: acme_kit::text::normalize(&flag_evaluation.kind),
    };
    CertificateAuthorityCalculationSink { client: certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient::new(channel) }.deliver(record).await
}

/// What `replay_flag_evaluation_id` keeps of each ReassignFlagEvaluation response.
#[derive(Clone, Debug, Default)]
pub struct FlagEvaluationIdRecord {
    pub flag_evaluation_id: String,
    pub message: String,
    pub kind: String,
}

/// Where `replay_flag_evaluation_id` hands each record.
trait FlagEvaluationIdSink {
    fn deliver(&self, record: FlagEvaluationIdRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CertificateAuthorityCalculationService`.
struct CertificateAuthorityCalculationSink {
    client:
        certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient<
            Channel,
        >,
}

impl FlagEvaluationIdSink for CertificateAuthorityCalculationSink {
    fn deliver(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_certificate_authority_options(certificates::CompareCertificateAuthorityOptionsRequest {
                    details: Some(certificates::compare_certificate_authority_options_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    certificate_authority_id: record.flag_evaluation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
