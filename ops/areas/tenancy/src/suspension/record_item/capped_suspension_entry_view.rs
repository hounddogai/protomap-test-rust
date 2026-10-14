use super::*;

use acme_kit::text;

/// How screens and exports show a capped suspension entry.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionEntryView {
    pub code_text: String,
    pub title: String,
    pub unit: String,
    pub ttl_seconds_text: String,
    pub timezone: String,
    pub sequence: String,
    pub cluster_text: String,
    pub namespace: String,
    pub name_text: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_capped_suspension_entry_view(item: &CappedSuspensionEntry) -> CappedSuspensionEntryView {
    CappedSuspensionEntryView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        unit: text::truncate(&item.unit, 80),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        timezone: item.timezone.clone(),
        sequence: item.sequence.to_string(),
        cluster_text: item.cluster.clone(),
        namespace: item.namespace.clone(),
        name_text: item.name.clone(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_suspension_entry_views(items: &[CappedSuspensionEntry]) -> Vec<CappedSuspensionEntryView> {
    let mut views: Vec<CappedSuspensionEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_suspension_entry_view(item));
    }
    views
}

pub fn sum_capped_suspension_entry_view(view: &CappedSuspensionEntryView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_sequence as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_suspension_entry("code-874", "title-52");
        let view = to_capped_suspension_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_suspension_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
