use super::*;

use acme_kit::text;

/// How screens and exports show a delegation draft.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDraftView {
    pub code: String,
    pub name_text: String,
    pub priority: String,
    pub maximum_text: String,
    pub quantity_text: String,
    pub segment: String,
    pub updated_at: String,
    pub tenant_id: String,
    pub category: String,
    pub ttl_seconds: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_delegation_draft_view(item: &DelegationDraft) -> DelegationDraftView {
    DelegationDraftView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        priority: item.priority.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        quantity_text: item.quantity.to_string(),
        segment: item.segment.clone(),
        updated_at: item.updated_at.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 120),
        category: text::truncate(&item.category, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_draft_views(items: &[DelegationDraft]) -> Vec<DelegationDraftView> {
    let mut views: Vec<DelegationDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_draft_view(item));
    }
    views
}

pub fn sum_delegation_draft_view(view: &DelegationDraftView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_priority as f64 + parsed_maximum + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_draft("code-922", "name-789");
        let view = to_delegation_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
