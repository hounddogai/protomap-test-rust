use super::*;

use acme_kit::text;

/// How screens and exports show a legacy flow log variance.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFlowLogVarianceView {
    pub code: String,
    pub title: String,
    pub grace_days_text: String,
    pub unit_text: String,
    pub cipher_text: String,
    pub contact_email: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_legacy_flow_log_variance_view(item: &LegacyFlowLogVariance) -> LegacyFlowLogVarianceView {
    LegacyFlowLogVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        grace_days_text: item.grace_days.to_string(),
        unit_text: item.unit.clone(),
        cipher_text: item.cipher.clone(),
        contact_email: text::truncate(&item.contact_email, 80),
        segment: text::truncate(&item.segment, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_flow_log_variance_views(items: &[LegacyFlowLogVariance]) -> Vec<LegacyFlowLogVarianceView> {
    let mut views: Vec<LegacyFlowLogVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_flow_log_variance_view(item));
    }
    views
}

pub fn sum_legacy_flow_log_variance_view(view: &LegacyFlowLogVarianceView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_flow_log_variance("code-926", "title-339");
        let view = to_legacy_flow_log_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_flow_log_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
