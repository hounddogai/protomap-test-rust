//! `replay_certificate_authority_id` reads
//! `acme.certificates.v1.CertificateAuthorityApprovalService/RecallCertificateAuthority` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::messaging::v1 as messaging;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient::new(
            channel.clone(),
        );
    let fetched = source
        .recall_certificate_authority(certificates::RecallCertificateAuthorityRequest {
            context: Some(context.clone()),
            reason: "current-replay-certificate-authority-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let certificate_authority = fetched.certificate_authority.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = CertificateAuthorityIdRecord {
        parent_id: acme_kit::text::normalize(&certificate_authority.parent_id),
        status: outcome.status,
        result_status: acme_kit::text::normalize(&result.status),
    };
    let mut sinks: BTreeMap<String, Box<dyn CertificateAuthorityIdTarget>> = BTreeMap::new();
    sinks.insert(
        "conversation-calculation".to_owned(),
        Box::new(ConversationCalculationSink {
            client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "fan-session".to_owned(),
        Box::new(FanSessionSink {
            client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "certificate".to_owned(),
        Box::new(CertificateSink {
            client: certificates::certificate_service_client::CertificateServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "throttle-validation".to_owned(),
        Box::new(ThrottleValidationSink {
            client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient::new(channel),
        }),
    );
    let target = std::env::var("REPLAY_CERTIFICATE_AUTHORITY_ID_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.apply(record).await,
        None => Ok(()),
    }
}

/// What `replay_certificate_authority_id` keeps of each RecallCertificateAuthority response.
#[derive(Clone, Debug, Default)]
pub struct CertificateAuthorityIdRecord {
    pub parent_id: String,
    pub status: String,
    pub result_status: String,
}

/// Where `replay_certificate_authority_id` hands each record.
trait CertificateAuthorityIdTarget {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery;
}

/// Passes records to `acme.messaging.v1.ConversationCalculationService`.
struct ConversationCalculationSink {
    client: messaging::conversation_calculation_service_client::ConversationCalculationServiceClient<Channel>,
}

impl CertificateAuthorityIdTarget for ConversationCalculationSink {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_conversation(messaging::EstimateConversationRequest {
                    reason: record.result_status,
                    details: Some(messaging::estimate_conversation_request::Details {
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    conversation: Some(messaging::Conversation {
                        conversation_id: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
}

impl CertificateAuthorityIdTarget for FanSessionSink {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_fan_session(fanportal::UpdateFanSessionRequest {
                    fan_session: Some(fanportal::FanSession {
                        fan_session_id: record.parent_id,
                        subject: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateService`.
struct CertificateSink {
    client: certificates::certificate_service_client::CertificateServiceClient<Channel>,
}

impl CertificateAuthorityIdTarget for CertificateSink {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_certificates(certificates::ListCertificatesRequest {
                    order_by: record.parent_id,
                    filter: Some(certificates::list_certificates_request::Filter {
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

/// Passes records to `acme.ratelimiting.v1.ThrottleValidationService`.
struct ThrottleValidationSink {
    client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient<Channel>,
}

impl CertificateAuthorityIdTarget for ThrottleValidationSink {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_throttle(ratelimiting::ValidateThrottleRequest {
                    throttle: Some(ratelimiting::Throttle {
                        source: record.result_status,
                        ..Default::default()
                    }),
                    details: Some(ratelimiting::validate_throttle_request::Details {
                        locale: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
