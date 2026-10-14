use super::*;

/// How screens and exports show a plan manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanManifestView {
    pub code: String,
    pub title: String,
    pub minimum_text: String,
    pub budget: String,
    pub category: String,
    pub region: String,
    pub quota_units: String,
    pub cost_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_plan_manifest_view(item: &PlanManifest) -> PlanManifestView {
    PlanManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        budget: format!("{:.2}", item.budget),
        category: item.category.clone(),
        region: item.region.clone(),
        quota_units: item.quota_units.to_string(),
        cost_text: format!("{:.2}", item.cost),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_manifest_views(items: &[PlanManifest]) -> Vec<PlanManifestView> {
    let mut views: Vec<PlanManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_manifest_view(item));
    }
    views
}

pub fn sum_plan_manifest_view(view: &PlanManifestView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_minimum + parsed_budget + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_manifest("code-241", "title-533");
        let view = to_plan_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
