use super::*;

use acme_kit::text;

/// How screens and exports show a strict probe estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictProbeEstimateView {
    pub code_text: String,
    pub title_text: String,
    pub version: String,
    pub host_text: String,
    pub minimum: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_strict_probe_estimate_view(item: &StrictProbeEstimate) -> StrictProbeEstimateView {
    StrictProbeEstimateView {
        code_text: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        version: item.version.to_string(),
        host_text: item.host.clone(),
        minimum: format!("{:.2}", item.minimum),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_probe_estimate_views(items: &[StrictProbeEstimate]) -> Vec<StrictProbeEstimateView> {
    let mut views: Vec<StrictProbeEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_probe_estimate_view(item));
    }
    views
}

pub fn sum_strict_probe_estimate_view(view: &StrictProbeEstimateView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_version as f64 + parsed_minimum + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_probe_estimate("code-17", "title-424");
        let view = to_strict_probe_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_probe_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
