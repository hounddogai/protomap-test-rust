use super::*;

use acme_kit::text;

/// How screens and exports show a regional allowlist target.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistTargetView {
    pub code: String,
    pub label_text: String,
    pub active_text: String,
    pub revision: String,
    pub rotation_days: String,
    pub minimum: String,
    pub owner_id: String,
    pub host: String,
    pub issuer: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_regional_allowlist_target_view(item: &RegionalAllowlistTarget) -> RegionalAllowlistTargetView {
    RegionalAllowlistTargetView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        active_text: format!("{}", item.active),
        revision: item.revision.to_string(),
        rotation_days: item.rotation_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        owner_id: item.owner_id.clone(),
        host: item.host.clone(),
        issuer: text::truncate(&item.issuer, 80),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_allowlist_target_views(items: &[RegionalAllowlistTarget]) -> Vec<RegionalAllowlistTargetView> {
    let mut views: Vec<RegionalAllowlistTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_allowlist_target_view(item));
    }
    views
}

pub fn sum_regional_allowlist_target_view(view: &RegionalAllowlistTargetView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_rotation_days as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_allowlist_target("code-679", "label-905");
        let view = to_regional_allowlist_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_allowlist_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
