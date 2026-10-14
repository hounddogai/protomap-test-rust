use super::*;

use acme_kit::text;

/// How screens and exports show a action item claim.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemClaimView {
    pub code_text: String,
    pub label: String,
    pub created_at: String,
    pub audience_text: String,
    pub checksum: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_action_item_claim_view(item: &ActionItemClaim) -> ActionItemClaimView {
    ActionItemClaimView {
        code_text: text::truncate(&item.code, 80),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        audience_text: item.audience.clone(),
        checksum: item.checksum.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_claim_views(items: &[ActionItemClaim]) -> Vec<ActionItemClaimView> {
    let mut views: Vec<ActionItemClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_claim_view(item));
    }
    views
}

pub fn sum_action_item_claim_view(view: &ActionItemClaimView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_claim("code-242", "label-281");
        let view = to_action_item_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
