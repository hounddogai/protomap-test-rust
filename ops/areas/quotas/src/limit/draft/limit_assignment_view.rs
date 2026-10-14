use super::*;

use acme_kit::text;

/// How screens and exports show a limit assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub checksum_text: String,
    pub cluster: String,
    pub margin_text: String,
    pub unit: String,
    pub ttl_seconds_text: String,
    pub reviewer: String,
    pub namespace: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_limit_assignment_view(item: &LimitAssignment) -> LimitAssignmentView {
    LimitAssignmentView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        cost: format!("{:.2}", item.cost),
        checksum_text: text::truncate(&item.checksum, 120),
        cluster: item.cluster.clone(),
        margin_text: format!("{:.2}", item.margin),
        unit: text::truncate(&item.unit, 80),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        reviewer: text::truncate(&item.reviewer, 120),
        namespace: text::truncate(&item.namespace, 64),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_assignment_views(items: &[LimitAssignment]) -> Vec<LimitAssignmentView> {
    let mut views: Vec<LimitAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_assignment_view(item));
    }
    views
}

pub fn sum_limit_assignment_view(view: &LimitAssignmentView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_cost + parsed_margin + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_assignment("code-227", "label-608");
        let view = to_limit_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
