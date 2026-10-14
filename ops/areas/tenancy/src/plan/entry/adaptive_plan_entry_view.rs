use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive plan entry.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanEntryView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub owner_id: String,
    pub issuer: String,
    pub reviewer_text: String,
    pub key_id: String,
    pub limit_text: String,
    pub segment_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_adaptive_plan_entry_view(item: &AdaptivePlanEntry) -> AdaptivePlanEntryView {
    AdaptivePlanEntryView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 64),
        ends_at: item.ends_at.to_string(),
        owner_id: item.owner_id.clone(),
        issuer: item.issuer.clone(),
        reviewer_text: item.reviewer.clone(),
        key_id: text::truncate(&item.key_id, 120),
        limit_text: item.limit.to_string(),
        segment_text: item.segment.clone(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_plan_entry_views(items: &[AdaptivePlanEntry]) -> Vec<AdaptivePlanEntryView> {
    let mut views: Vec<AdaptivePlanEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_plan_entry_view(item));
    }
    views
}

pub fn sum_adaptive_plan_entry_view(view: &AdaptivePlanEntryView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_plan_entry("code-877", "name-73");
        let view = to_adaptive_plan_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_plan_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
