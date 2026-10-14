use super::*;

use acme_kit::text;

/// How screens and exports show a regional runbook entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRunbookEntryView {
    pub code: String,
    pub name_text: String,
    pub namespace: String,
    pub currency: String,
    pub sequence: String,
    pub maximum: String,
    pub cost: String,
    pub visible: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_regional_runbook_entry_view(item: &RegionalRunbookEntry) -> RegionalRunbookEntryView {
    RegionalRunbookEntryView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        namespace: item.namespace.clone(),
        currency: text::truncate(&item.currency, 120),
        sequence: item.sequence.to_string(),
        maximum: format!("{:.2}", item.maximum),
        cost: format!("{:.2}", item.cost),
        visible: format!("{}", item.visible),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_runbook_entry_views(items: &[RegionalRunbookEntry]) -> Vec<RegionalRunbookEntryView> {
    let mut views: Vec<RegionalRunbookEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_runbook_entry_view(item));
    }
    views
}

pub fn sum_regional_runbook_entry_view(view: &RegionalRunbookEntryView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_maximum + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_runbook_entry("code-787", "name-90");
        let view = to_regional_runbook_entry_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_runbook_entry_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
