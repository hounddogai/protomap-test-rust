//! Infrastructure shared by the platform domains. Business decisions stay in the domain crates.

pub mod auth;
pub mod codec;
pub mod config;
mod handlers;
pub mod journal;
pub mod ports;
pub mod transport;

use acme_proto::acme::common::v1::RequestContext;

/// Check the tenancy envelope before the workflow can store data or call another service.
pub fn validate(context: Option<&RequestContext>) -> Result<(), tonic::Status> {
    let context = context.ok_or_else(|| tonic::Status::invalid_argument("context is required"))?;
    if context.tenant_id.trim().is_empty() {
        return Err(tonic::Status::invalid_argument("context.tenant_id is required"));
    }
    match context.actor.as_ref() {
        Some(actor) if actor.service_account || !actor.user_id.is_empty() => Ok(()),
        _ => Err(tonic::Status::unauthenticated("context.actor is required")),
    }
}
