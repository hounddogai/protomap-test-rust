use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled identity bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityBundleView {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub priority: String,
    pub amount_text: String,
    pub unit: String,
    pub label_text: String,
    pub status_label: String,
}

pub fn to_scheduled_identity_bundle_view(item: &ScheduledIdentityBundle) -> ScheduledIdentityBundleView {
    ScheduledIdentityBundleView {
        code: item.code.clone(),
        name: item.name.clone(),
        external_ref: text::truncate(&item.external_ref, 120),
        priority: item.priority.to_string(),
        amount_text: format!("{:.2}", item.amount),
        unit: text::truncate(&item.unit, 40),
        label_text: text::truncate(&item.label, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_identity_bundle_views(items: &[ScheduledIdentityBundle]) -> Vec<ScheduledIdentityBundleView> {
    let mut views: Vec<ScheduledIdentityBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_identity_bundle_view(item));
    }
    views
}

pub fn sum_scheduled_identity_bundle_view(view: &ScheduledIdentityBundleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_priority as f64 + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_identity_bundle("code-587", "name-378");
        let view = to_scheduled_identity_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_identity_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
