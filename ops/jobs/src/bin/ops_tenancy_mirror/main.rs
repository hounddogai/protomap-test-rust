//! Command ops_tenancy_mirror mirrors tenancy's records between the services that hold them.

mod route_crypto_key_message;
mod settle_lineage_nodes_status;
mod mirror_pending_certificate_authorities_id;
mod hand_off_venue_id;
mod carry_key_rings_message;
mod settle_seat_maps_message;
mod push_delivery_notes;
mod carry_box_office_sale_number;
mod settle_config_set_calculation_id;
mod copy_config_change_rates_id;
mod propagate_patient_home_id;
mod sync_gateway_policy_report_uri;
mod backfill_ledger_account_id;
mod hand_off_crypto_key_message;
mod relay_config_set_id;
mod carry_tours_status;
mod carry_key_ring_reports_id;
mod export_config_changes_id;
mod propagate_gateway_policy_event_message;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = route_crypto_key_message::run(channel.clone(), context.clone()).await {
        eprintln!("route crypto key message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_lineage_nodes_status::run(channel.clone(), context.clone()).await {
        eprintln!("settle lineage nodes status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_pending_certificate_authorities_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror pending certificate authorities id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_venue_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off venue id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_key_rings_message::run(channel.clone(), context.clone()).await {
        eprintln!("carry key rings message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_seat_maps_message::run(channel.clone(), context.clone()).await {
        eprintln!("settle seat maps message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_delivery_notes::run(channel.clone(), context.clone()).await {
        eprintln!("push delivery notes failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_box_office_sale_number::run(channel.clone(), context.clone()).await {
        eprintln!("carry box office sale number failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_config_set_calculation_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle config set calculation id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_config_change_rates_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy config change rates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_patient_home_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate patient home id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_gateway_policy_report_uri::run(channel.clone(), context.clone()).await {
        eprintln!("sync gateway policy report uri failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_ledger_account_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill ledger account id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_crypto_key_message::run(channel.clone(), context.clone()).await {
        eprintln!("hand off crypto key message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_config_set_id::run(channel.clone(), context.clone()).await {
        eprintln!("relay config set id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_tours_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry tours status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_key_ring_reports_id::run(channel.clone(), context.clone()).await {
        eprintln!("carry key ring reports id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_config_changes_id::run(channel.clone(), context.clone()).await {
        eprintln!("export config changes id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_gateway_policy_event_message::run(channel.clone(), context.clone()).await {
        eprintln!("propagate gateway policy event message failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
