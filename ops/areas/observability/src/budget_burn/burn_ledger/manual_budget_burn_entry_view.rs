use super::*;

use acme_kit::text;

/// How screens and exports show a manual budget burn entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnEntryView {
    pub code_text: String,
    pub title: String,
    pub quantity: String,
    pub ttl_seconds: String,
    pub created_at_text: String,
    pub channel: String,
    pub rotation_days: String,
    pub cluster: String,
    pub cipher_text: String,
    pub key_id: String,
    pub external_ref_text: String,
    pub status_label: String,
}

pub fn to_manual_budget_burn_entry_view(item: &ManualBudgetBurnEntry) -> ManualBudgetBurnEntryView {
    ManualBudgetBurnEntryView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        quantity: item.quantity.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        created_at_text: item.created_at.to_string(),
        channel: item.channel.clone(),
        rotation_days: item.rotation_days.to_string(),
        cluster: item.cluster.clone(),
        cipher_text: text::truncate(&item.cipher, 120),
        key_id: text::truncate(&item.key_id, 120),
        external_ref_text: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_budget_burn_entry_views(items: &[ManualBudgetBurnEntry]) -> Vec<ManualBudgetBurnEntryView> {
    let mut views: Vec<ManualBudgetBurnEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_budget_burn_entry_view(item));
    }
    views
}

pub fn sum_manual_budget_burn_entry_view(view: &ManualBudgetBurnEntryView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_ttl_seconds as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_budget_burn_entry("code-633", "title-652");
        let view = to_manual_budget_burn_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_budget_burn_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
