use super::*;

use acme_kit::text;

/// How screens and exports show a fallback verifier estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierEstimateView {
    pub code: String,
    pub name_text: String,
    pub description: String,
    pub batch_size: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_fallback_verifier_estimate_view(item: &FallbackVerifierEstimate) -> FallbackVerifierEstimateView {
    FallbackVerifierEstimateView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        description: item.description.clone(),
        batch_size: item.batch_size.to_string(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_verifier_estimate_views(items: &[FallbackVerifierEstimate]) -> Vec<FallbackVerifierEstimateView> {
    let mut views: Vec<FallbackVerifierEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_verifier_estimate_view(item));
    }
    views
}

pub fn sum_fallback_verifier_estimate_view(view: &FallbackVerifierEstimateView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_verifier_estimate("code-552", "name-30");
        let view = to_fallback_verifier_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_verifier_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
