use super::*;

/// How screens and exports show a default forecast policy.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastPolicyView {
    pub code: String,
    pub name: String,
    pub capacity_text: String,
    pub contact_email: String,
    pub scope: String,
    pub key_id_text: String,
    pub expires_at: String,
    pub threshold_text: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_default_forecast_policy_view(item: &DefaultForecastPolicy) -> DefaultForecastPolicyView {
    DefaultForecastPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        capacity_text: item.capacity.to_string(),
        contact_email: item.contact_email.clone(),
        scope: item.scope.clone(),
        key_id_text: item.key_id.clone(),
        expires_at: item.expires_at.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_forecast_policy_views(items: &[DefaultForecastPolicy]) -> Vec<DefaultForecastPolicyView> {
    let mut views: Vec<DefaultForecastPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_forecast_policy_view(item));
    }
    views
}

pub fn sum_default_forecast_policy_view(view: &DefaultForecastPolicyView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_expires_at as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_forecast_policy("code-96", "name-271");
        let view = to_default_forecast_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_forecast_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
