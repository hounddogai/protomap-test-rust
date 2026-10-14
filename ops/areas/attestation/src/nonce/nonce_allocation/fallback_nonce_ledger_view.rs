use super::*;

use acme_kit::text;

/// How screens and exports show a fallback nonce ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceLedgerView {
    pub code: String,
    pub name: String,
    pub description: String,
    pub priority: String,
    pub region_text: String,
    pub owner_id_text: String,
    pub country: String,
    pub grace_days_text: String,
    pub label: String,
    pub audience: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_fallback_nonce_ledger_view(item: &FallbackNonceLedger) -> FallbackNonceLedgerView {
    FallbackNonceLedgerView {
        code: item.code.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        priority: item.priority.to_string(),
        region_text: text::truncate(&item.region, 120),
        owner_id_text: item.owner_id.clone(),
        country: text::truncate(&item.country, 64),
        grace_days_text: item.grace_days.to_string(),
        label: text::truncate(&item.label, 80),
        audience: item.audience.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_nonce_ledger_views(items: &[FallbackNonceLedger]) -> Vec<FallbackNonceLedgerView> {
    let mut views: Vec<FallbackNonceLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_nonce_ledger_view(item));
    }
    views
}

pub fn sum_fallback_nonce_ledger_view(view: &FallbackNonceLedgerView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_grace_days as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_nonce_ledger("code-471", "name-438");
        let view = to_fallback_nonce_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_nonce_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
