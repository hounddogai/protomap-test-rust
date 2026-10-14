//! `propagate_patient_home_id` reads `acme.patientapp.v1.PatientHomeQueryService/PeekPatientHome` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = patientapp::patient_home_query_service_client::PatientHomeQueryServiceClient::new(channel.clone());
    let reply = source
        .peek_patient_home(patientapp::PeekPatientHomeRequest {
            context: Some(context.clone()),
            patient_home_id: "main-propagate-patient-home-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let patient_home = reply.patient_home.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = PatientHomeIdRecord {
        patient_home_id: patient_home.patient_home_id,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn PatientHomeIdSink>> = BTreeMap::new();
    sinks.insert(
        "key-ring-approval",
        Box::new(KeyRingApprovalSink {
            client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "gateway-policy-query",
        Box::new(GatewayPolicyQuerySink {
            client: edgegateway::gateway_policy_query_service_client::GatewayPolicyQueryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "box-office-sale-query",
        Box::new(BoxOfficeSaleQuerySink {
            client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "rate-limit-reporting",
        Box::new(RateLimitReportingSink {
            client: ratelimiting::rate_limit_reporting_service_client::RateLimitReportingServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("box-office-sale-query") else {
        anyhow::bail!("no sink registered for box-office-sale-query");
    };
    sink.forward(record).await
}

/// What `propagate_patient_home_id` keeps of each PeekPatientHome response.
#[derive(Clone, Debug, Default)]
pub struct PatientHomeIdRecord {
    pub patient_home_id: String,
    pub message: String,
}

/// Where `propagate_patient_home_id` hands each record.
trait PatientHomeIdSink {
    fn forward(&self, record: PatientHomeIdRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRingApprovalService`.
struct KeyRingApprovalSink {
    client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl PatientHomeIdSink for KeyRingApprovalSink {
    fn forward(&self, record: PatientHomeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::GetKeyRingApprovalStatusRequest {
                    key_ring_id: record.patient_home_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_key_ring_approval_status(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.edgegateway.v1.GatewayPolicyQueryService`.
struct GatewayPolicyQuerySink {
    client: edgegateway::gateway_policy_query_service_client::GatewayPolicyQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl PatientHomeIdSink for GatewayPolicyQuerySink {
    fn forward(&self, record: PatientHomeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(edgegateway::SuggestGatewayPoliciesRequest {
                    page: Some(common::PageRequest {
                        page_token: record.patient_home_id,
                        ..Default::default()
                    }),
                    filter: Some(edgegateway::suggest_gateway_policies_request::Filter {
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_gateway_policies(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.boxoffice.v1.BoxOfficeSaleQueryService`.
struct BoxOfficeSaleQuerySink {
    client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient<Channel>,
}

impl PatientHomeIdSink for BoxOfficeSaleQuerySink {
    fn forward(&self, record: PatientHomeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_box_office_sales(boxoffice::FindBoxOfficeSalesRequest {
                    filter: Some(boxoffice::find_box_office_sales_request::Filter {
                        region: record.patient_home_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.RateLimitReportingService`.
struct RateLimitReportingSink {
    client: ratelimiting::rate_limit_reporting_service_client::RateLimitReportingServiceClient<Channel>,
}

impl PatientHomeIdSink for RateLimitReportingSink {
    fn forward(&self, record: PatientHomeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .share_rate_limit_report(ratelimiting::ShareRateLimitReportRequest {
                    details: Some(ratelimiting::share_rate_limit_report_request::Details {
                        template_id: record.patient_home_id,
                        channel: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
