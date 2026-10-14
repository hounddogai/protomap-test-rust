//! `export_venues_status` reads `acme.venues.v1.VenueQueryService/CountVenues` and hands what it keeps to
//! `acme.certificates.v1.CertificateAuthorityWorkflowService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = venues::venue_query_service_client::VenueQueryServiceClient::new(channel.clone());
    let reply = source
        .count_venues(venues::CountVenuesRequest {
            context: Some(context.clone()),
            order_by: "primary-export-venues-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = VenuesStatusRecord {
        status: result.status,
        message: result.message,
    };
    CertificateAuthorityWorkflowSink {
        client:
            certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient::new(
                channel,
            ),
    }
    .deliver(record)
    .await
}

/// What `export_venues_status` keeps of each CountVenues response.
#[derive(Clone, Debug, Default)]
pub struct VenuesStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `export_venues_status` hands each record.
trait VenuesStatusSink {
    fn deliver(&self, record: VenuesStatusRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CertificateAuthorityWorkflowService`.
struct CertificateAuthorityWorkflowSink {
    client:
        certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient<Channel>,
}

impl VenuesStatusSink for CertificateAuthorityWorkflowSink {
    fn deliver(&self, record: VenuesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_certificate_authority_workflow(certificates::StartCertificateAuthorityWorkflowRequest {
                    certificate_authority: Some(certificates::CertificateAuthority {
                        name: record.status,
                        parent_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
