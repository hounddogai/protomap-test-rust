//! Command ops_netpolicy_backfill backfills netpolicy's records between the services that hold them.

mod route_work_order_code;
mod mirror_sponsors_timezone;
mod relay_config_change_id;
mod backfill_audit_event_decision_id;
mod forward_documents_id;
mod relay_purchase_requisitions_status;
mod route_forum_status;
mod settle_config_set_workflow_id;
mod copy_quota_bucket_totals_id;
mod forward_flag_rollout_preferences_message;
mod push_config_change_status;
mod relay_rate_limit_options_code;
mod carry_shows_message;
mod replay_anomaly_status;
mod backfill_permission_check_eligibility_status;
mod export_seat_map_id;
mod replay_certificate_authority_id;
mod forward_gateway_policy_report_status;
mod propagate_flag_rollout_message;
mod propagate_quota_bucket_id;
mod propagate_forum_id;
mod sync_secret_version_workflow_kind;
mod settle_secret_workflow_message;
mod relay_feature_flag_message;
mod sync_crypto_key_workflow_message;
mod forward_feature_flags_status;
mod forward_seat_maps_id;
mod settle_secret_versions_id;
mod sync_purchase_requisitions_status;
mod push_work_order_calculation_message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = route_work_order_code::run(channel.clone(), context.clone()).await {
        eprintln!("route work order code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_sponsors_timezone::run(channel.clone(), context.clone()).await {
        eprintln!("mirror sponsors timezone failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_config_change_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay config change id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_audit_event_decision_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill audit event decision id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_documents_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward documents id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_purchase_requisitions_status::run(channel.clone(), context.clone()).await {
        eprintln!("relay purchase requisitions status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_forum_status::run(channel.clone(), context.clone()).await {
        eprintln!("route forum status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_config_set_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle config set workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_quota_bucket_totals_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy quota bucket totals id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_flag_rollout_preferences_message::run(channel.clone(), context.clone()).await {
        eprintln!("forward flag rollout preferences message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_config_change_status::run(channel.clone(), context.clone()).await {
        eprintln!("push config change status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_rate_limit_options_code::run(channel.clone(), context.clone()).await {
        eprintln!("relay rate limit options code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_shows_message::run(channel.clone(), context.clone()).await {
        eprintln!("carry shows message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_anomaly_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay anomaly status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_permission_check_eligibility_status::run(channel.clone(), context.clone()).await {
        eprintln!("backfill permission check eligibility status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_seat_map_id::run(channel.clone(), context.clone()).await {
        eprintln!("export seat map id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_certificate_authority_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay certificate authority id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_gateway_policy_report_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward gateway policy report status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_flag_rollout_message::run(channel.clone(), context.clone()).await {
        eprintln!("propagate flag rollout message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_quota_bucket_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate quota bucket id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_forum_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate forum id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_secret_version_workflow_kind::run(channel.clone(), context.clone()).await {
        eprintln!("sync secret version workflow kind failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_secret_workflow_message::run(channel.clone(), context.clone()).await {
        eprintln!("settle secret workflow message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_feature_flag_message::run(channel.clone(), context.clone()).await {
        eprintln!("relay feature flag message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_crypto_key_workflow_message::run(channel.clone(), context.clone()).await {
        eprintln!("sync crypto key workflow message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_feature_flags_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward feature flags status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_seat_maps_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward seat maps id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_secret_versions_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle secret versions id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_purchase_requisitions_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync purchase requisitions status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_work_order_calculation_message::run(channel.clone(), context.clone()).await {
        eprintln!("push work order calculation message failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
