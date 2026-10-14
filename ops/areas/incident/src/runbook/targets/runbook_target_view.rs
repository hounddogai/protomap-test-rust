use super::*;

use acme_kit::text;

/// How screens and exports show a runbook target.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTargetView {
    pub code: String,
    pub title: String,
    pub checksum_text: String,
    pub priority: String,
    pub quota_units: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_runbook_target_view(item: &RunbookTarget) -> RunbookTargetView {
    RunbookTargetView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        checksum_text: item.checksum.clone(),
        priority: item.priority.to_string(),
        quota_units: item.quota_units.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_target_views(items: &[RunbookTarget]) -> Vec<RunbookTargetView> {
    let mut views: Vec<RunbookTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_target_view(item));
    }
    views
}

pub fn sum_runbook_target_view(view: &RunbookTargetView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_target("code-663", "title-293");
        let view = to_runbook_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
