//! Command ops_workload_relay relays workload's records between the services that hold them.

mod copy_form_options_code;
mod hand_off_flag_rollout_versions_message;
mod sync_price_lists_status;
mod export_show_id;
mod export_fan_sessions_id;
mod carry_search_index_eligibility_status;
mod replay_admissions_id;
mod push_certificate_status;
mod relay_config_change_approval_message;
mod hand_off_certificates_id;
mod carry_config_set_message;
mod propagate_yard_id;
mod carry_stock_item_message;
mod mirror_invoice_batch_id;
mod backfill_crypto_key_workflow_id;
mod relay_users_status;
mod settle_csr_decision_id;
mod settle_notifications_token;
mod hand_off_config_changes_status;
mod route_user_decision_status;
mod mirror_warehouse_status;
mod replay_flag_evaluation_message;
mod forward_certificate_authorities_message;
mod hand_off_audit_events_token;
mod sync_api_key_status;
mod push_subscriptions_token;
mod replay_audit_events_status;
mod propagate_admissions_status;
mod forward_sellers_id;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    let mut failed = 0;
    if let Err(error) = copy_form_options_code::run(channel.clone(), context.clone()).await {
        eprintln!("copy form options code failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_flag_rollout_versions_message::run(channel.clone(), context.clone()).await {
        eprintln!("hand off flag rollout versions message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_price_lists_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync price lists status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_show_id::run(channel.clone(), context.clone()).await {
        eprintln!("export show id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = export_fan_sessions_id::run(channel.clone(), context.clone()).await {
        eprintln!("export fan sessions id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_search_index_eligibility_status::run(channel.clone(), context.clone()).await {
        eprintln!("carry search index eligibility status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_admissions_id::run(channel.clone(), context.clone()).await {
        eprintln!("replay admissions id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_certificate_status::run(channel.clone(), context.clone()).await {
        eprintln!("push certificate status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_config_change_approval_message::run(channel.clone(), context.clone()).await {
        eprintln!("relay config change approval message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_certificates_id::run(channel.clone(), context.clone()).await {
        eprintln!("hand off certificates id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_config_set_message::run(channel.clone(), context.clone()).await {
        eprintln!("carry config set message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_yard_id::run(channel.clone(), context.clone()).await {
        eprintln!("propagate yard id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = carry_stock_item_message::run(channel.clone(), context.clone()).await {
        eprintln!("carry stock item message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_invoice_batch_id::run(channel.clone(), context.clone()).await {
        eprintln!("mirror invoice batch id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = backfill_crypto_key_workflow_id::run(channel.clone(), context.clone()).await {
        eprintln!("backfill crypto key workflow id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = relay_users_status::run(channel.clone(), context.clone()).await {
        eprintln!("relay users status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_csr_decision_id::run(channel.clone(), context.clone()).await {
        eprintln!("settle csr decision id failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = settle_notifications_token::run(channel.clone(), context.clone()).await {
        eprintln!("settle notifications token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_config_changes_status::run(channel.clone(), context.clone()).await {
        eprintln!("hand off config changes status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = route_user_decision_status::run(channel.clone(), context.clone()).await {
        eprintln!("route user decision status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = mirror_warehouse_status::run(channel.clone(), context.clone()).await {
        eprintln!("mirror warehouse status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_flag_evaluation_message::run(channel.clone(), context.clone()).await {
        eprintln!("replay flag evaluation message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_certificate_authorities_message::run(channel.clone(), context.clone()).await {
        eprintln!("forward certificate authorities message failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = hand_off_audit_events_token::run(channel.clone(), context.clone()).await {
        eprintln!("hand off audit events token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = sync_api_key_status::run(channel.clone(), context.clone()).await {
        eprintln!("sync api key status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = push_subscriptions_token::run(channel.clone(), context.clone()).await {
        eprintln!("push subscriptions token failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = replay_audit_events_status::run(channel.clone(), context.clone()).await {
        eprintln!("replay audit events status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = propagate_admissions_status::run(channel.clone(), context.clone()).await {
        eprintln!("propagate admissions status failed: {error:#}");
        failed += 1;
    }
    if let Err(error) = forward_sellers_id::run(channel.clone(), context.clone()).await {
        eprintln!("forward sellers id failed: {error:#}");
        failed += 1;
    }
    if failed > 0 {
        anyhow::bail!("{failed} jobs failed");
    }
    Ok(())
}
