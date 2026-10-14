use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBaselineView {
    pub code_text: String,
    pub title: String,
    pub sequence_text: String,
    pub namespace: String,
    pub weight: String,
    pub contact_email_text: String,
    pub margin: String,
    pub updated_at_text: String,
    pub summary_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_dashboard_baseline_view(item: &DashboardBaseline) -> DashboardBaselineView {
    DashboardBaselineView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        sequence_text: item.sequence.to_string(),
        namespace: text::truncate(&item.namespace, 64),
        weight: format!("{:.2}", item.weight),
        contact_email_text: item.contact_email.clone(),
        margin: format!("{:.2}", item.margin),
        updated_at_text: item.updated_at.to_string(),
        summary_text: item.summary.clone(),
        region: text::truncate(&item.region, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_baseline_views(items: &[DashboardBaseline]) -> Vec<DashboardBaselineView> {
    let mut views: Vec<DashboardBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_baseline_view(item));
    }
    views
}

pub fn sum_dashboard_baseline_view(view: &DashboardBaselineView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_weight + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_dashboard_baseline("code-327", "title-841");
        let view = to_dashboard_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_dashboard_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
