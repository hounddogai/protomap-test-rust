//! `forward_certificate_authorities_id` reads
//! `acme.certificates.v1.CertificateAuthorityCalculationService/RecalculateCertificateAuthorities` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::patientapp::v1 as patientapp;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient::new(channel.clone());
    let fetched = source
        .recalculate_certificate_authorities(certificates::RecalculateCertificateAuthoritiesRequest {
            context: Some(context.clone()),
            certificate_authority_id: "default-forward-certificate-authorities-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let certificate_authority = fetched.certificate_authority.unwrap_or_default();
    let record = CertificateAuthoritiesIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        name: certificate_authority.name,
        parent_id: acme_kit::text::normalize(&certificate_authority.parent_id),
    };
    let sinks: Vec<(&str, Box<dyn CertificateAuthoritiesIdTarget>)> = vec![
        ("certificate-authority-validation", Box::new(CertificateAuthorityValidationSink { client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient::new(channel.clone()) })),
        ("patient-home", Box::new(PatientHomeSink { client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel.clone()), context: context.clone() })),
        ("chat-room", Box::new(ChatRoomSink { client: chat::chat_room_service_client::ChatRoomServiceClient::new(channel), context: context.clone() })),
    ];
    for (kind, sink) in &sinks {
        if *kind != "patient-home" {
            continue;
        }
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `forward_certificate_authorities_id` keeps of each RecalculateCertificateAuthorities response.
#[derive(Clone, Debug, Default)]
pub struct CertificateAuthoritiesIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub name: String,
    pub parent_id: String,
}

/// Where `forward_certificate_authorities_id` hands each record.
trait CertificateAuthoritiesIdTarget {
    fn write(&self, record: CertificateAuthoritiesIdRecord) -> Delivery;
}

/// Writes records to `acme.certificates.v1.CertificateAuthorityValidationService`.
struct CertificateAuthorityValidationSink {
    client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient<
        Channel,
    >,
}

impl CertificateAuthoritiesIdTarget for CertificateAuthorityValidationSink {
    fn write(&self, record: CertificateAuthoritiesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_certificate_authority(certificates::ValidateCertificateAuthorityRequest {
                    reason: record.correlation_id,
                    certificate_authority: Some(certificates::CertificateAuthority {
                        name: record.status,
                        parent_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
    context: common::RequestContext,
}

impl CertificateAuthoritiesIdTarget for PatientHomeSink {
    fn write(&self, record: CertificateAuthoritiesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(patientapp::GetPatientHomeRequest {
                    patient_home_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_patient_home(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.chat.v1.ChatRoomService`.
struct ChatRoomSink {
    client: chat::chat_room_service_client::ChatRoomServiceClient<Channel>,
    context: common::RequestContext,
}

impl CertificateAuthoritiesIdTarget for ChatRoomSink {
    fn write(&self, record: CertificateAuthoritiesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(chat::ListChatRoomsRequest {
                    filter: Some(chat::list_chat_rooms_request::Filter {
                        region: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_chat_rooms(request).await?;
            Ok(())
        })
    }
}
