use super::*;

use acme_kit::text;

/// How screens and exports show a binding ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedgerView {
    pub code_text: String,
    pub label: String,
    pub minimum: String,
    pub title: String,
    pub ttl_seconds: String,
    pub currency: String,
    pub pinned: String,
    pub timezone: String,
    pub phone: String,
    pub duration_minutes: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_binding_ledger_view(item: &BindingLedger) -> BindingLedgerView {
    BindingLedgerView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 120),
        minimum: format!("{:.2}", item.minimum),
        title: text::truncate(&item.title, 40),
        ttl_seconds: item.ttl_seconds.to_string(),
        currency: text::truncate(&item.currency, 80),
        pinned: format!("{}", item.pinned),
        timezone: item.timezone.clone(),
        phone: item.phone.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_ledger_views(items: &[BindingLedger]) -> Vec<BindingLedgerView> {
    let mut views: Vec<BindingLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_ledger_view(item));
    }
    views
}

pub fn sum_binding_ledger_view(view: &BindingLedgerView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_minimum + (parsed_ttl_seconds as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_ledger("code-526", "label-670");
        let view = to_binding_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
