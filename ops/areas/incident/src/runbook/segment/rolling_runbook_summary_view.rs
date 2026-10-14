use super::*;

use acme_kit::text;

/// How screens and exports show a rolling runbook summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookSummaryView {
    pub code: String,
    pub title_text: String,
    pub discount_text: String,
    pub rotation_days: String,
    pub owner_id_text: String,
    pub weight: String,
    pub phone: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_rolling_runbook_summary_view(item: &RollingRunbookSummary) -> RollingRunbookSummaryView {
    RollingRunbookSummaryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        rotation_days: item.rotation_days.to_string(),
        owner_id_text: item.owner_id.clone(),
        weight: format!("{:.2}", item.weight),
        phone: text::truncate(&item.phone, 40),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_runbook_summary_views(items: &[RollingRunbookSummary]) -> Vec<RollingRunbookSummaryView> {
    let mut views: Vec<RollingRunbookSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_runbook_summary_view(item));
    }
    views
}

pub fn sum_rolling_runbook_summary_view(view: &RollingRunbookSummaryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_discount + (parsed_rotation_days as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_runbook_summary("code-929", "title-361");
        let view = to_rolling_runbook_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_runbook_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
