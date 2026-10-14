//! Command ops_netpolicy_reconciler reconcilers netpolicy's records between the services that hold them.

mod sync_deliveries_status;
mod copy_flag_rollouts_status;
mod hand_off_deliveries_id;
mod export_broadcasts_id;
mod export_key_ring_id;
mod hand_off_show_title;
mod propagate_checkout_sessions_timezone;
mod route_fulfillments_token;
mod mirror_ledger_account_decision_code;
mod hand_off_broadcasts_message;
mod sync_policies_message;
mod backfill_role_binding_export_notes;
mod push_throttle_kind;
mod route_feature_flag_updates_name;
mod propagate_environment_workflow_id;
mod carry_patients_message;
mod forward_certificate_authority_id;
mod export_labeling_task_subject;
mod export_key_ring_message;
mod copy_config_change_message;
mod relay_api_key_workflow_id;
mod mirror_feature_flag_version_name;
mod forward_role_binding_id;
mod replay_broadcasts_message;
mod backfill_flag_evaluation_version_id;
mod relay_yard_rates_status;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = sync_deliveries_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync deliveries status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_flag_rollouts_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy flag rollouts status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_deliveries_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off deliveries id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_broadcasts_id::run(channel.clone(), context.clone()).await {
        eprintln!("export broadcasts id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_key_ring_id::run(channel.clone(), context.clone()).await {
        eprintln!("export key ring id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_show_title::run(channel.clone(), context.clone()).await {
        eprintln!("hand off show title failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_checkout_sessions_timezone::run(channel.clone(), context.clone()).await {
        eprintln!("propagate checkout sessions timezone failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_fulfillments_token::run(channel.clone(), context.clone()).await {
        eprintln!("route fulfillments token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_ledger_account_decision_code::run(channel.clone(), context.clone()).await {
        eprintln!("mirror ledger account decision code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_broadcasts_message::run(channel.clone(), context.clone()).await {
        eprintln!("hand off broadcasts message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_policies_message::run(channel.clone(), context.clone()).await {
        eprintln!("sync policies message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_role_binding_export_notes::run(channel.clone(), context.clone()).await {
        eprintln!("backfill role binding export notes failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_throttle_kind::run(channel.clone(), context.clone()).await {
        eprintln!("push throttle kind failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_feature_flag_updates_name::run(channel.clone(), context.clone()).await {
        eprintln!("route feature flag updates name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_environment_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate environment workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_patients_message::run(channel.clone(), context.clone()).await {
        eprintln!("carry patients message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_certificate_authority_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward certificate authority id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_labeling_task_subject::run(channel.clone(), context.clone()).await {
        eprintln!("export labeling task subject failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_key_ring_message::run(channel.clone(), context.clone()).await {
        eprintln!("export key ring message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_config_change_message::run(channel.clone(), context.clone()).await {
        eprintln!("copy config change message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_api_key_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay api key workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_feature_flag_version_name::run(channel.clone(), context.clone()).await {
        eprintln!("mirror feature flag version name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_role_binding_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward role binding id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_broadcasts_message::run(channel.clone(), context.clone()).await {
        eprintln!("replay broadcasts message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_flag_evaluation_version_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill flag evaluation version id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_yard_rates_status::run(channel.clone(), context.clone()).await {
        eprintln!("relay yard rates status failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
