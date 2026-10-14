use super::*;

use acme_kit::text;

/// How screens and exports show a primary tenant score.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScoreView {
    pub code_text: String,
    pub title_text: String,
    pub checksum: String,
    pub quota_units: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_primary_tenant_score_view(item: &PrimaryTenantScore) -> PrimaryTenantScoreView {
    PrimaryTenantScoreView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        checksum: text::truncate(&item.checksum, 120),
        quota_units: item.quota_units.to_string(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_tenant_score_views(items: &[PrimaryTenantScore]) -> Vec<PrimaryTenantScoreView> {
    let mut views: Vec<PrimaryTenantScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_tenant_score_view(item));
    }
    views
}

pub fn sum_primary_tenant_score_view(view: &PrimaryTenantScoreView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_tenant_score("code-538", "title-615");
        let view = to_primary_tenant_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_tenant_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
