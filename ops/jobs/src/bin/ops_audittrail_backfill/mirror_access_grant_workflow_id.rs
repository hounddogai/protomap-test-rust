//! `mirror_access_grant_workflow_id` reads
//! `acme.secrets.v1.AccessGrantWorkflowService/PauseAccessGrantWorkflow` and hands what it keeps to the sink
//! of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient::new(channel.clone());
    let found = source
        .pause_access_grant_workflow(secrets::PauseAccessGrantWorkflowRequest {
            context: Some(context.clone()),
            reason: "main-mirror-access-grant-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let access_grant = found.access_grant.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = AccessGrantWorkflowIdRecord {
        access_grant_id: access_grant.access_grant_id,
        reference_id: outcome.reference_id,
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AccessGrantWorkflowIdTarget>> = BTreeMap::new();
    sinks.insert(
        "chat-room",
        Box::new(ChatRoomSink {
            client: chat::chat_room_service_client::ChatRoomServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "patient-home",
        Box::new(PatientHomeSink {
            client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("chat-room") else {
        anyhow::bail!("no sink registered for chat-room");
    };
    sink.push(record).await
}

/// What `mirror_access_grant_workflow_id` keeps of each PauseAccessGrantWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct AccessGrantWorkflowIdRecord {
    pub access_grant_id: String,
    pub reference_id: String,
    pub correlation_id: String,
}

/// Where `mirror_access_grant_workflow_id` hands each record.
trait AccessGrantWorkflowIdTarget {
    fn push(&self, record: AccessGrantWorkflowIdRecord) -> Delivery;
}

/// Writes records to `acme.chat.v1.ChatRoomService`.
struct ChatRoomSink {
    client: chat::chat_room_service_client::ChatRoomServiceClient<Channel>,
}

impl AccessGrantWorkflowIdTarget for ChatRoomSink {
    fn push(&self, record: AccessGrantWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_chat_room(chat::UpdateChatRoomRequest {
                    chat_room: Some(chat::ChatRoom {
                        chat_room_id: record.access_grant_id,
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
}

impl AccessGrantWorkflowIdTarget for PatientHomeSink {
    fn push(&self, record: AccessGrantWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_patient_home(patientapp::ClonePatientHomeRequest {
                    patient_home: Some(patientapp::PatientHome {
                        patient_home_id: record.access_grant_id,
                        subject: record.correlation_id,
                        ..Default::default()
                    }),
                    request_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
