use super::*;

/// How screens and exports show a verifier forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierForecastView {
    pub code_text: String,
    pub label_text: String,
    pub floor_value_text: String,
    pub cluster: String,
    pub limit: String,
    pub batch_size: String,
    pub channel: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_verifier_forecast_view(item: &VerifierForecast) -> VerifierForecastView {
    VerifierForecastView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        cluster: item.cluster.clone(),
        limit: item.limit.to_string(),
        batch_size: item.batch_size.to_string(),
        channel: item.channel.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_forecast_views(items: &[VerifierForecast]) -> Vec<VerifierForecastView> {
    let mut views: Vec<VerifierForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_forecast_view(item));
    }
    views
}

pub fn sum_verifier_forecast_view(view: &VerifierForecastView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_floor_value + (parsed_limit as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_forecast("code-868", "label-248");
        let view = to_verifier_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
