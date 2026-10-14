//! Command ops_attestation_sync syncs attestation's records between the services that hold them.

mod backfill_fuel_card_message;
mod hand_off_csr_workflow_message;
mod export_dataset_name;
mod forward_certificate_authorities_token;
mod carry_config_set_rules_status;
mod export_certificate_options_notes;
mod copy_labeling_task_id;
mod push_api_key_workflow_code;
mod propagate_rate_limit_status;
mod settle_key_ring_id;
mod copy_prescription_id;
mod replay_audit_event_status;
mod backfill_box_office_sale_id;
mod hand_off_quota_buckets_code;
mod hand_off_feature_flag_status;
mod settle_contract_id;
mod backfill_labeling_tasks_id;
mod relay_key_rotation_decision_status;
mod settle_notifications_id;
mod relay_metric_series_eligibility_code;
mod hand_off_venue_eligibility_id;
mod forward_fixed_assets_status;
mod replay_throttle_id;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = backfill_fuel_card_message::run(channel.clone(), context.clone()).await {
        eprintln!("backfill fuel card message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_csr_workflow_message::run(channel.clone(), context.clone()).await {
        eprintln!("hand off csr workflow message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_dataset_name::run(channel.clone(), context.clone()).await {
        eprintln!("export dataset name failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_certificate_authorities_token::run(channel.clone(), context.clone()).await {
        eprintln!("forward certificate authorities token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_config_set_rules_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry config set rules status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_certificate_options_notes::run(channel.clone(), context.clone()).await {
        eprintln!("export certificate options notes failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_labeling_task_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy labeling task id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_api_key_workflow_code::run(channel.clone(), context.clone()).await {
        eprintln!("push api key workflow code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_rate_limit_status::run(channel.clone(), context.clone()).await {
        eprintln!("propagate rate limit status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_key_ring_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle key ring id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = copy_prescription_id::run(channel.clone(), context.clone()).await {
        eprintln!("copy prescription id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_audit_event_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay audit event status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_box_office_sale_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill box office sale id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_quota_buckets_code::run(channel.clone(), context.clone()).await {
        eprintln!("hand off quota buckets code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_feature_flag_status::run(channel.clone(), context.clone()).await {
        eprintln!("hand off feature flag status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_contract_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle contract id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_labeling_tasks_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill labeling tasks id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_key_rotation_decision_status::run(channel.clone(), context.clone()).await {
        eprintln!("relay key rotation decision status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_notifications_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle notifications id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_metric_series_eligibility_code::run(channel.clone(), context.clone()).await {
        eprintln!("relay metric series eligibility code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_venue_eligibility_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off venue eligibility id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_fixed_assets_status::run(channel.clone(), context.clone()).await {
        eprintln!("forward fixed assets status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_throttle_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay throttle id failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
