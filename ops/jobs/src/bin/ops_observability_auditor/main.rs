//! Command ops_observability_auditor auditors observability's records between the services that hold them.

mod replay_incidents_message;
mod route_timesheets_id;
mod replay_config_changes_status;
mod route_throttle_kind;
mod push_cash_position_message;
mod forward_api_key_updates_message;
mod settle_flag_rollout_test_notice_status;
mod route_config_set_approval_status_name;
mod copy_throttle_import_status_source;
mod relay_flag_rollout_export_id;
mod sync_seat_map_name;
mod push_chat_room_id;
mod relay_get_feature_flags_id;
mod route_warehouse_id;
mod carry_permission_check_export_status;
mod route_access_policy_eligibility_id;
mod settle_access_grant_description;
mod route_config_set_report_id;
mod settle_admissions_status;
mod forward_invoice_batch_notes;
mod propagate_metric_series_id;
mod route_role_binding_message;
mod push_feature_flag_id;
mod replay_csrs_subject;
mod hand_off_flag_evaluation_export_source;
mod mirror_get_broadcasts_id;
mod backfill_role_bindings_id;
mod carry_pending_csrs_status;
mod carry_notification_status;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = replay_incidents_message::run(channel.clone(), context.clone()).await {
        eprintln!("replay incidents message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_timesheets_id::run(channel.clone(), context.clone()).await {
        eprintln!("route timesheets id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_config_changes_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay config changes status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_throttle_kind::run(channel.clone(), context.clone()).await {
        eprintln!("route throttle kind failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_cash_position_message::run(channel.clone(), context.clone()).await {
        eprintln!("push cash position message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_api_key_updates_message::run(channel.clone(), context.clone()).await {
        eprintln!("forward api key updates message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_flag_rollout_test_notice_status::run(channel.clone(), context.clone()).await {
        eprintln!("settle flag rollout test notice status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_config_set_approval_status_name::run(channel.clone(), context.clone()).await {
        eprintln!("route config set approval status name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_throttle_import_status_source::run(channel.clone(), context.clone()).await {
        eprintln!("copy throttle import status source failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_flag_rollout_export_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay flag rollout export id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_seat_map_name::run(channel.clone(), context.clone()).await {
        eprintln!("sync seat map name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_chat_room_id::run(channel.clone(), context.clone()).await {
        eprintln!("push chat room id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_get_feature_flags_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay get feature flags id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_warehouse_id::run(channel.clone(), context.clone()).await {
        eprintln!("route warehouse id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_permission_check_export_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry permission check export status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_access_policy_eligibility_id::run(channel.clone(), context.clone()).await {
        eprintln!("route access policy eligibility id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_access_grant_description::run(channel.clone(), context.clone()).await {
        eprintln!("settle access grant description failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_config_set_report_id::run(channel.clone(), context.clone()).await {
        eprintln!("route config set report id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_admissions_status::run(channel.clone(), context.clone()).await {
        eprintln!("settle admissions status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_invoice_batch_notes::run(channel.clone(), context.clone()).await {
        eprintln!("forward invoice batch notes failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_metric_series_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate metric series id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_role_binding_message::run(channel.clone(), context.clone()).await {
        eprintln!("route role binding message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_feature_flag_id::run(channel.clone(), context.clone()).await {
        eprintln!("push feature flag id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_csrs_subject::run(channel.clone(), context.clone()).await {
        eprintln!("replay csrs subject failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_flag_evaluation_export_source::run(channel.clone(), context.clone()).await {
        eprintln!("hand off flag evaluation export source failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_get_broadcasts_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror get broadcasts id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_role_bindings_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill role bindings id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_pending_csrs_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry pending csrs status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_notification_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry notification status failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
