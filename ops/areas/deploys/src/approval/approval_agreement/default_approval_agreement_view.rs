use super::*;

use acme_kit::text;

/// How screens and exports show a default approval agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultApprovalAgreementView {
    pub code: String,
    pub title: String,
    pub ends_at: String,
    pub approved: String,
    pub window_days: String,
    pub starts_at: String,
    pub label_text: String,
    pub margin: String,
    pub summary: String,
    pub ratio: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_default_approval_agreement_view(item: &DefaultApprovalAgreement) -> DefaultApprovalAgreementView {
    DefaultApprovalAgreementView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        approved: format!("{}", item.approved),
        window_days: item.window_days.to_string(),
        starts_at: item.starts_at.to_string(),
        label_text: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        summary: text::truncate(&item.summary, 80),
        ratio: format!("{:.2}", item.ratio),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_approval_agreement_views(items: &[DefaultApprovalAgreement]) -> Vec<DefaultApprovalAgreementView> {
    let mut views: Vec<DefaultApprovalAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_approval_agreement_view(item));
    }
    views
}

pub fn sum_default_approval_agreement_view(view: &DefaultApprovalAgreementView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_window_days as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_approval_agreement("code-402", "title-446");
        let view = to_default_approval_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_approval_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
