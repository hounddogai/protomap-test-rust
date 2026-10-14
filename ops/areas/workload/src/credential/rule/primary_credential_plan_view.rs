use super::*;

use acme_kit::text;

/// How screens and exports show a primary credential plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCredentialPlanView {
    pub code_text: String,
    pub label: String,
    pub checksum_text: String,
    pub quota_units_text: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_primary_credential_plan_view(item: &PrimaryCredentialPlan) -> PrimaryCredentialPlanView {
    PrimaryCredentialPlanView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        checksum_text: item.checksum.clone(),
        quota_units_text: item.quota_units.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_credential_plan_views(items: &[PrimaryCredentialPlan]) -> Vec<PrimaryCredentialPlanView> {
    let mut views: Vec<PrimaryCredentialPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_credential_plan_view(item));
    }
    views
}

pub fn sum_primary_credential_plan_view(view: &PrimaryCredentialPlanView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_credential_plan("code-628", "label-913");
        let view = to_primary_credential_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_credential_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
