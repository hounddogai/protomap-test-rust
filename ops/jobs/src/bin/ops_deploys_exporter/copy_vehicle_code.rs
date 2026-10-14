//! `copy_vehicle_code` reads `acme.fleet.v1.VehicleQueryService/LookupVehicle` and hands what it keeps to
//! `acme.keymanagement.v1.KeyRotationApprovalService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel.clone());
    let fetched = source
        .lookup_vehicle(fleet::LookupVehicleRequest {
            context: Some(context.clone()),
            vehicle_id: "current-copy-vehicle-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let vehicle = fetched.vehicle.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = VehicleCodeRecord {
        code: acme_kit::text::normalize(&vehicle.code),
        name: vehicle.name,
        status: acme_kit::text::normalize(&result.status),
    };
    KeyRotationApprovalSink {
        client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient::new(channel),
        context: context.clone(),
    }
    .apply(record)
    .await
}

/// What `copy_vehicle_code` keeps of each LookupVehicle response.
#[derive(Clone, Debug, Default)]
pub struct VehicleCodeRecord {
    pub code: String,
    pub name: String,
    pub status: String,
}

/// Where `copy_vehicle_code` hands each record.
trait VehicleCodeDestination {
    fn apply(&self, record: VehicleCodeRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRotationApprovalService`.
struct KeyRotationApprovalSink {
    client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl VehicleCodeDestination for KeyRotationApprovalSink {
    fn apply(&self, record: VehicleCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::SubmitKeyRotationRequest {
                    key_rotation: Some(keymanagement::KeyRotation {
                        title: record.status,
                        ..Default::default()
                    }),
                    details: Some(keymanagement::submit_key_rotation_request::Details {
                        comment: record.code,
                        ..Default::default()
                    }),
                    key_rotation_id: record.name,
                    ..Default::default()
                }),
                &context,
            )?;
            client.submit_key_rotation(request).await?;
            Ok(())
        })
    }
}
