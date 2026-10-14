use super::*;

use acme_kit::text;

/// How screens and exports show a grant agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAgreementView {
    pub code: String,
    pub label_text: String,
    pub region_text: String,
    pub window_days: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_grant_agreement_view(item: &GrantAgreement) -> GrantAgreementView {
    GrantAgreementView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        region_text: item.region.clone(),
        window_days: item.window_days.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_agreement_views(items: &[GrantAgreement]) -> Vec<GrantAgreementView> {
    let mut views: Vec<GrantAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_agreement_view(item));
    }
    views
}

pub fn sum_grant_agreement_view(view: &GrantAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_agreement("code-177", "label-519");
        let view = to_grant_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
