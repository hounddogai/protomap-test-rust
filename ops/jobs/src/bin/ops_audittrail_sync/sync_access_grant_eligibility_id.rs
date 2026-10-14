//! `sync_access_grant_eligibility_id` reads
//! `acme.secrets.v1.AccessGrantValidationService/CheckAccessGrantEligibility` and hands what it keeps to
//! `acme.fleet.v1.VehicleService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(channel.clone());
    let found = source
        .check_access_grant_eligibility(secrets::CheckAccessGrantEligibilityRequest {
            context: Some(context.clone()),
            reason: "main-sync-access-grant-eligibility-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let access_grant = found.access_grant.unwrap_or_default();
    let record = AccessGrantEligibilityIdRecord {
        reference_id: outcome.reference_id,
        status: acme_kit::text::normalize(&outcome.status),
        description: access_grant.description,
    };
    VehicleSink {
        client: fleet::vehicle_service_client::VehicleServiceClient::new(channel),
        context: context.clone(),
    }
    .write(record)
    .await
}

/// What `sync_access_grant_eligibility_id` keeps of each CheckAccessGrantEligibility response.
#[derive(Clone, Debug, Default)]
pub struct AccessGrantEligibilityIdRecord {
    pub reference_id: String,
    pub status: String,
    pub description: String,
}

/// Where `sync_access_grant_eligibility_id` hands each record.
trait AccessGrantEligibilityIdSink {
    fn write(&self, record: AccessGrantEligibilityIdRecord) -> Delivery;
}

/// Passes records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
    context: common::RequestContext,
}

impl AccessGrantEligibilityIdSink for VehicleSink {
    fn write(&self, record: AccessGrantEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::GetVehicleRequest {
                    vehicle_id: record.reference_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_vehicle(request).await?;
            Ok(())
        })
    }
}
