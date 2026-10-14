//! Command ops_incident_relay relays incident's records between the services that hold them.

mod backfill_throttle_workflow_id;
mod sync_pay_run_status;
mod backfill_conversation_options_status;
mod sync_feature_flag_version_id;
mod forward_seat_map_id;
mod replay_api_key_source_message;
mod replay_flag_evaluation_id;
mod forward_appointments_status;
mod export_venue_rules_name;
mod propagate_flag_evaluation_source;
mod route_api_keys_token;
mod copy_fuel_cards_status;
mod copy_yard_status;
mod backfill_config_set_workflow_id;
mod forward_config_change_lifecycle_id;
mod sync_price_list_message;
mod push_throttle_report_id;
mod settle_deliveries_message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = backfill_throttle_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill throttle workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_pay_run_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync pay run status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_conversation_options_status::run(channel.clone(), context.clone()).await {
        eprintln!("backfill conversation options status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_feature_flag_version_id::run(channel.clone(), context.clone()).await {
        eprintln!("sync feature flag version id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_seat_map_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward seat map id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_api_key_source_message::run(channel.clone(), context.clone()).await {
        eprintln!("replay api key source message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_flag_evaluation_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay flag evaluation id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_appointments_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward appointments status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_venue_rules_name::run(channel.clone(), context.clone()).await {
        eprintln!("export venue rules name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_flag_evaluation_source::run(channel.clone(), context.clone()).await {
        eprintln!("propagate flag evaluation source failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_api_keys_token::run(channel.clone(), context.clone()).await {
        eprintln!("route api keys token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_fuel_cards_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy fuel cards status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_yard_status::run(channel.clone(), context.clone()).await {
        eprintln!("copy yard status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_config_set_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill config set workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_config_change_lifecycle_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward config change lifecycle id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_price_list_message::run(channel.clone(), context.clone()).await {
        eprintln!("sync price list message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_throttle_report_id::run(channel.clone(), context.clone()).await {
        eprintln!("push throttle report id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_deliveries_message::run(channel.clone(), context.clone()).await {
        eprintln!("settle deliveries message failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
