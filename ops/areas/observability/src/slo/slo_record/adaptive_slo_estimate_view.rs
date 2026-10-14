use super::*;

/// How screens and exports show a adaptive slo estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloEstimateView {
    pub code_text: String,
    pub label_text: String,
    pub source_text: String,
    pub locale_text: String,
    pub duration_minutes: String,
    pub port: String,
    pub margin: String,
    pub starts_at_text: String,
    pub weight: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_adaptive_slo_estimate_view(item: &AdaptiveSloEstimate) -> AdaptiveSloEstimateView {
    AdaptiveSloEstimateView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        source_text: item.source.clone(),
        locale_text: item.locale.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        port: item.port.to_string(),
        margin: format!("{:.2}", item.margin),
        starts_at_text: item.starts_at.to_string(),
        weight: format!("{:.2}", item.weight),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_slo_estimate_views(items: &[AdaptiveSloEstimate]) -> Vec<AdaptiveSloEstimateView> {
    let mut views: Vec<AdaptiveSloEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_slo_estimate_view(item));
    }
    views
}

pub fn sum_adaptive_slo_estimate_view(view: &AdaptiveSloEstimateView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_port as f64) + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_slo_estimate("code-327", "label-956");
        let view = to_adaptive_slo_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_slo_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
