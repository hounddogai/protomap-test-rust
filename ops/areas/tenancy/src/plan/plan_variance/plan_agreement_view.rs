use super::*;

use acme_kit::text;

/// How screens and exports show a plan agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAgreementView {
    pub code_text: String,
    pub title_text: String,
    pub segment: String,
    pub phone: String,
    pub notes: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_plan_agreement_view(item: &PlanAgreement) -> PlanAgreementView {
    PlanAgreementView {
        code_text: text::truncate(&item.code, 80),
        title_text: text::truncate(&item.title, 64),
        segment: text::truncate(&item.segment, 120),
        phone: text::truncate(&item.phone, 64),
        notes: item.notes.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_agreement_views(items: &[PlanAgreement]) -> Vec<PlanAgreementView> {
    let mut views: Vec<PlanAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_agreement_view(item));
    }
    views
}

pub fn sum_plan_agreement_view(view: &PlanAgreementView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_agreement("code-630", "title-231");
        let view = to_plan_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
