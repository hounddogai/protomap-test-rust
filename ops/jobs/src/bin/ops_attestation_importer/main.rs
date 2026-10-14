//! Command ops_attestation_importer importers attestation's records between the services that hold them.

mod copy_crypto_keys_token;
mod copy_csr_message;
mod settle_search_index_rules_code;
mod carry_gateway_policies_id;
mod carry_certificate_id;
mod replay_flag_evaluation_id;
mod route_video_visit_rates_id;
mod relay_admission_options_code;
mod propagate_pay_runs_message;
mod forward_pending_key_rings_id;
mod copy_rate_limits_status;
mod relay_get_patient_homes_id;
mod forward_fan_session_id;
mod relay_documents_status;
mod hand_off_certificate_workflow_number;
mod copy_config_sets_message;
mod sync_role_binding_export_number;
mod backfill_certificates_message;
mod forward_key_rotation_workflow_status;
mod route_video_visit_title;
mod mirror_sponsor_options_message;
mod export_dispatch_order_notes;
mod propagate_warehouse_name;
mod replay_gateway_policy_updates_description;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = copy_crypto_keys_token::run(channel.clone(), context.clone()).await {
        eprintln!("copy crypto keys token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_csr_message::run(channel.clone(), context.clone()).await {
        eprintln!("copy csr message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_search_index_rules_code::run(channel.clone(), context.clone()).await {
        eprintln!("settle search index rules code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_gateway_policies_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry gateway policies id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_certificate_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry certificate id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_flag_evaluation_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay flag evaluation id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_video_visit_rates_id::run(channel.clone(), context.clone()).await {
        eprintln!("route video visit rates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_admission_options_code::run(channel.clone(), context.clone()).await {
        eprintln!("relay admission options code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_pay_runs_message::run(channel.clone(), context.clone()).await {
        eprintln!("propagate pay runs message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_pending_key_rings_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward pending key rings id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_rate_limits_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy rate limits status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_get_patient_homes_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay get patient homes id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_fan_session_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward fan session id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_documents_status::run(channel.clone(), context.clone()).await {
        eprintln!("relay documents status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_certificate_workflow_number::run(channel.clone(), context.clone()).await {
        eprintln!("hand off certificate workflow number failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_config_sets_message::run(channel.clone(), context.clone()).await {
        eprintln!("copy config sets message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_role_binding_export_number::run(channel.clone(), context.clone()).await {
        eprintln!("sync role binding export number failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_certificates_message::run(channel.clone(), context.clone()).await {
        eprintln!("backfill certificates message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_key_rotation_workflow_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward key rotation workflow status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_video_visit_title::run(channel.clone(), context.clone()).await {
        eprintln!("route video visit title failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_sponsor_options_message::run(channel.clone(), context.clone()).await {
        eprintln!("mirror sponsor options message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_dispatch_order_notes::run(channel.clone(), context.clone()).await {
        eprintln!("export dispatch order notes failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_warehouse_name::run(channel.clone(), context.clone()).await {
        eprintln!("propagate warehouse name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_gateway_policy_updates_description::run(channel.clone(), context.clone()).await {
        eprintln!("replay gateway policy updates description failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
