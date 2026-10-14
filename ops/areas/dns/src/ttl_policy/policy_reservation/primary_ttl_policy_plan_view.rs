use super::*;

use acme_kit::text;

/// How screens and exports show a primary ttl policy plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyPlanView {
    pub code: String,
    pub title_text: String,
    pub source_text: String,
    pub summary: String,
    pub fingerprint: String,
    pub contact_email_text: String,
    pub approved: String,
    pub score: String,
    pub maximum_text: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_primary_ttl_policy_plan_view(item: &PrimaryTtlPolicyPlan) -> PrimaryTtlPolicyPlanView {
    PrimaryTtlPolicyPlanView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        source_text: text::truncate(&item.source, 80),
        summary: text::truncate(&item.summary, 120),
        fingerprint: item.fingerprint.clone(),
        contact_email_text: item.contact_email.clone(),
        approved: format!("{}", item.approved),
        score: format!("{:.2}", item.score),
        maximum_text: format!("{:.2}", item.maximum),
        label: text::truncate(&item.label, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_ttl_policy_plan_views(items: &[PrimaryTtlPolicyPlan]) -> Vec<PrimaryTtlPolicyPlanView> {
    let mut views: Vec<PrimaryTtlPolicyPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_ttl_policy_plan_view(item));
    }
    views
}

pub fn sum_primary_ttl_policy_plan_view(view: &PrimaryTtlPolicyPlanView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_score + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_ttl_policy_plan("code-493", "title-974");
        let view = to_primary_ttl_policy_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_ttl_policy_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
