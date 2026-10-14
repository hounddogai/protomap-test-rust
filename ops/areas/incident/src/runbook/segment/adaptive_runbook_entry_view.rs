use super::*;

/// How screens and exports show a adaptive runbook entry.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookEntryView {
    pub code: String,
    pub title_text: String,
    pub rate: String,
    pub rotation_days: String,
    pub limit: String,
    pub ratio: String,
    pub maximum_text: String,
    pub namespace: String,
    pub created_at: String,
    pub external_ref_text: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_adaptive_runbook_entry_view(item: &AdaptiveRunbookEntry) -> AdaptiveRunbookEntryView {
    AdaptiveRunbookEntryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        rotation_days: item.rotation_days.to_string(),
        limit: item.limit.to_string(),
        ratio: format!("{:.2}", item.ratio),
        maximum_text: format!("{:.2}", item.maximum),
        namespace: item.namespace.clone(),
        created_at: item.created_at.to_string(),
        external_ref_text: item.external_ref.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_runbook_entry_views(items: &[AdaptiveRunbookEntry]) -> Vec<AdaptiveRunbookEntryView> {
    let mut views: Vec<AdaptiveRunbookEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_runbook_entry_view(item));
    }
    views
}

pub fn sum_adaptive_runbook_entry_view(view: &AdaptiveRunbookEntryView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rate + (parsed_rotation_days as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_runbook_entry("code-140", "title-576");
        let view = to_adaptive_runbook_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_runbook_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
