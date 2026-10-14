//! Command ops_incident_sweeper sweepers incident's records between the services that hold them.

mod relay_fan_sessions_token;
mod hand_off_rate_limit_status;
mod copy_venue_decision_id;
mod mirror_users_id;
mod route_buildings_message;
mod replay_config_set_report_id;
mod carry_onboarding_plan_status;
mod route_broadcast_status;
mod push_flag_evaluation_id;
mod hand_off_route_binding_page_name;
mod replay_csr_message;
mod forward_config_change_report_id;
mod replay_ledger_account_message;
mod hand_off_key_ring_workflow_id;
mod settle_patient_id;
mod forward_cash_position_rates_id;
mod forward_crypto_key_workflows_status;
mod sync_config_change_id;
mod backfill_get_datasets_status;
mod backfill_admission_code;
mod hand_off_venue_code;
mod export_forum_id;
mod replay_access_policy_status;
mod push_deliveries_status;
mod hand_off_route_bindings_message;
mod hand_off_notifications_id;
mod propagate_key_ring_id;
mod replay_get_search_indexes_token;
mod push_performer_message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = relay_fan_sessions_token::run(channel.clone(), context.clone()).await {
        eprintln!("relay fan sessions token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_rate_limit_status::run(channel.clone(), context.clone()).await {
        eprintln!("hand off rate limit status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_venue_decision_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy venue decision id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_users_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror users id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_buildings_message::run(channel.clone(), context.clone()).await {
        eprintln!("route buildings message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_config_set_report_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay config set report id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_onboarding_plan_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry onboarding plan status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_broadcast_status::run(channel.clone(), context.clone()).await {
        eprintln!("route broadcast status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_flag_evaluation_id::run(channel.clone(), context.clone()).await {
        eprintln!("push flag evaluation id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_route_binding_page_name::run(channel.clone(), context.clone()).await {
        eprintln!("hand off route binding page name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_csr_message::run(channel.clone(), context.clone()).await {
        eprintln!("replay csr message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_config_change_report_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward config change report id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_ledger_account_message::run(channel.clone(), context.clone()).await {
        eprintln!("replay ledger account message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_key_ring_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off key ring workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_patient_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle patient id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_cash_position_rates_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward cash position rates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_crypto_key_workflows_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward crypto key workflows status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_config_change_id::run(channel.clone(), context.clone()).await {
        eprintln!("sync config change id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_get_datasets_status::run(channel.clone(), context.clone()).await {
        eprintln!("backfill get datasets status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_admission_code::run(channel.clone(), context.clone()).await {
        eprintln!("backfill admission code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_venue_code::run(channel.clone(), context.clone()).await {
        eprintln!("hand off venue code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_forum_id::run(channel.clone(), context.clone()).await {
        eprintln!("export forum id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_access_policy_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay access policy status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_deliveries_status::run(channel.clone(), context.clone()).await {
        eprintln!("push deliveries status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_route_bindings_message::run(channel.clone(), context.clone()).await {
        eprintln!("hand off route bindings message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_notifications_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off notifications id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_key_ring_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate key ring id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_get_search_indexes_token::run(channel.clone(), context.clone()).await {
        eprintln!("replay get search indexes token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_performer_message::run(channel.clone(), context.clone()).await {
        eprintln!("push performer message failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
