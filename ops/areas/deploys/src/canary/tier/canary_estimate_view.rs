use super::*;

/// How screens and exports show a canary estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEstimateView {
    pub code_text: String,
    pub title_text: String,
    pub score: String,
    pub weight: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_canary_estimate_view(item: &CanaryEstimate) -> CanaryEstimateView {
    CanaryEstimateView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        score: format!("{:.2}", item.score),
        weight: format!("{:.2}", item.weight),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_estimate_views(items: &[CanaryEstimate]) -> Vec<CanaryEstimateView> {
    let mut views: Vec<CanaryEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_estimate_view(item));
    }
    views
}

pub fn sum_canary_estimate_view(view: &CanaryEstimateView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_score + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_estimate("code-522", "title-139");
        let view = to_canary_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
