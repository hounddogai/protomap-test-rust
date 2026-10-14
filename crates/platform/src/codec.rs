//! Wire envelopes used by adapters crossing the platform's protobuf and JSON boundaries.

use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AuditEnvelope {
    pub trace_id: String,
    pub tenant_id: String,
}

pub fn json_payload(envelope: AuditEnvelope) -> Result<Vec<u8>, tonic::Status> {
    let encoded = serde_json::to_vec(&envelope).map_err(json_error)?;
    let decoded: AuditEnvelope = serde_json::from_slice(&encoded).map_err(json_error)?;
    serde_json::to_vec(&decoded).map_err(json_error)
}

fn json_error(error: serde_json::Error) -> tonic::Status {
    tracing::warn!(%error, "invalid JSON envelope");
    tonic::Status::invalid_argument("invalid JSON envelope")
}

/// Pack and unpack a typed message through Any, retaining the type URL at the boundary.
pub fn through_any<T: Message + Default>(message: &T, name: &str) -> Result<T, tonic::Status> {
    let envelope = prost_types::Any {
        type_url: format!("type.googleapis.com/{name}"),
        value: message.encode_to_vec(),
    };
    let wire = envelope.encode_to_vec();
    let decoded =
        prost_types::Any::decode(wire.as_slice()).map_err(|_| tonic::Status::data_loss("invalid Any envelope"))?;
    if decoded.type_url != format!("type.googleapis.com/{name}") {
        return Err(tonic::Status::invalid_argument("unexpected envelope type"));
    }
    T::decode(decoded.value.as_slice()).map_err(|_| tonic::Status::data_loss("invalid Any payload"))
}
