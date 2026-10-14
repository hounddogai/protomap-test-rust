use super::*;

use acme_kit::text;

/// How screens and exports show a pending policy item.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyItemView {
    pub code: String,
    pub name: String,
    pub discount_text: String,
    pub ceiling: String,
    pub checksum: String,
    pub pinned: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_pending_policy_item_view(item: &PendingPolicyItem) -> PendingPolicyItemView {
    PendingPolicyItemView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        discount_text: format!("{:.2}", item.discount),
        ceiling: format!("{:.2}", item.ceiling),
        checksum: item.checksum.clone(),
        pinned: format!("{}", item.pinned),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_policy_item_views(items: &[PendingPolicyItem]) -> Vec<PendingPolicyItemView> {
    let mut views: Vec<PendingPolicyItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_policy_item_view(item));
    }
    views
}

pub fn sum_pending_policy_item_view(view: &PendingPolicyItemView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_discount + parsed_ceiling + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_policy_item("code-225", "name-998");
        let view = to_pending_policy_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_policy_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
