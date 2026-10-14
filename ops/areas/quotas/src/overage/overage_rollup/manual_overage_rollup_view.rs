use super::*;

use acme_kit::text;

/// How screens and exports show a manual overage rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualOverageRollupView {
    pub code_text: String,
    pub title: String,
    pub minimum: String,
    pub priority_text: String,
    pub active: String,
    pub retries: String,
    pub segment: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_manual_overage_rollup_view(item: &ManualOverageRollup) -> ManualOverageRollupView {
    ManualOverageRollupView {
        code_text: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 120),
        minimum: format!("{:.2}", item.minimum),
        priority_text: item.priority.to_string(),
        active: format!("{}", item.active),
        retries: item.retries.to_string(),
        segment: item.segment.clone(),
        issuer_text: text::truncate(&item.issuer, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_overage_rollup_views(items: &[ManualOverageRollup]) -> Vec<ManualOverageRollupView> {
    let mut views: Vec<ManualOverageRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_overage_rollup_view(item));
    }
    views
}

pub fn sum_manual_overage_rollup_view(view: &ManualOverageRollupView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_minimum + (parsed_priority as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_overage_rollup("code-611", "title-236");
        let view = to_manual_overage_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_overage_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
