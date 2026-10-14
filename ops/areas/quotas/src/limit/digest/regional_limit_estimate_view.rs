use super::*;

/// How screens and exports show a regional limit estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalLimitEstimateView {
    pub code_text: String,
    pub label: String,
    pub phone: String,
    pub fingerprint_text: String,
    pub updated_at: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_regional_limit_estimate_view(item: &RegionalLimitEstimate) -> RegionalLimitEstimateView {
    RegionalLimitEstimateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        phone: item.phone.clone(),
        fingerprint_text: item.fingerprint.clone(),
        updated_at: item.updated_at.to_string(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_limit_estimate_views(items: &[RegionalLimitEstimate]) -> Vec<RegionalLimitEstimateView> {
    let mut views: Vec<RegionalLimitEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_limit_estimate_view(item));
    }
    views
}

pub fn sum_regional_limit_estimate_view(view: &RegionalLimitEstimateView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_updated_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_limit_estimate("code-315", "label-442");
        let view = to_regional_limit_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_limit_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
