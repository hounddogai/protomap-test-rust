use super::*;

use acme_kit::text;

/// How screens and exports show a binding summary.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSummaryView {
    pub code: String,
    pub label: String,
    pub weight_text: String,
    pub active: String,
    pub version: String,
    pub starts_at: String,
    pub amount: String,
    pub ceiling_text: String,
    pub ratio_text: String,
    pub margin: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_binding_summary_view(item: &BindingSummary) -> BindingSummaryView {
    BindingSummaryView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        weight_text: format!("{:.2}", item.weight),
        active: format!("{}", item.active),
        version: item.version.to_string(),
        starts_at: item.starts_at.to_string(),
        amount: format!("{:.2}", item.amount),
        ceiling_text: format!("{:.2}", item.ceiling),
        ratio_text: format!("{:.2}", item.ratio),
        margin: format!("{:.2}", item.margin),
        country: text::truncate(&item.country, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_summary_views(items: &[BindingSummary]) -> Vec<BindingSummaryView> {
    let mut views: Vec<BindingSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_summary_view(item));
    }
    views
}

pub fn sum_binding_summary_view(view: &BindingSummaryView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_weight + (parsed_version as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_summary("code-857", "label-744");
        let view = to_binding_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
