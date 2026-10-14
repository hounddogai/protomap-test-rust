use super::*;

use acme_kit::text;

/// How screens and exports show a approval summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalSummaryView {
    pub code: String,
    pub title_text: String,
    pub maximum: String,
    pub quantity: String,
    pub burst_text: String,
    pub country_text: String,
    pub phone_text: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_approval_summary_view(item: &ApprovalSummary) -> ApprovalSummaryView {
    ApprovalSummaryView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        maximum: format!("{:.2}", item.maximum),
        quantity: item.quantity.to_string(),
        burst_text: item.burst.to_string(),
        country_text: item.country.clone(),
        phone_text: item.phone.clone(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_summary_views(items: &[ApprovalSummary]) -> Vec<ApprovalSummaryView> {
    let mut views: Vec<ApprovalSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_summary_view(item));
    }
    views
}

pub fn sum_approval_summary_view(view: &ApprovalSummaryView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_maximum + (parsed_quantity as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_summary("code-1", "title-910");
        let view = to_approval_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
