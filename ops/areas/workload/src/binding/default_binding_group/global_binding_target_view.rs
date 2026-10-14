use super::*;

/// How screens and exports show a global binding target.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingTargetView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub country: String,
    pub fingerprint: String,
    pub host: String,
    pub limit: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_global_binding_target_view(item: &GlobalBindingTarget) -> GlobalBindingTargetView {
    GlobalBindingTargetView {
        code: item.code.clone(),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        country: item.country.clone(),
        fingerprint: item.fingerprint.clone(),
        host: item.host.clone(),
        limit: item.limit.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_binding_target_views(items: &[GlobalBindingTarget]) -> Vec<GlobalBindingTargetView> {
    let mut views: Vec<GlobalBindingTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_binding_target_view(item));
    }
    views
}

pub fn sum_global_binding_target_view(view: &GlobalBindingTargetView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_budget + (parsed_limit as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_binding_target("code-47", "title-222");
        let view = to_global_binding_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_binding_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
