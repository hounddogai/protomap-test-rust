use super::*;

use acme_kit::text;

/// How screens and exports show a lenient nonce baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceBaselineView {
    pub code_text: String,
    pub title: String,
    pub cluster_text: String,
    pub rate_text: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_lenient_nonce_baseline_view(item: &LenientNonceBaseline) -> LenientNonceBaselineView {
    LenientNonceBaselineView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cluster_text: text::truncate(&item.cluster, 80),
        rate_text: format!("{:.2}", item.rate),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_nonce_baseline_views(items: &[LenientNonceBaseline]) -> Vec<LenientNonceBaselineView> {
    let mut views: Vec<LenientNonceBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_nonce_baseline_view(item));
    }
    views
}

pub fn sum_lenient_nonce_baseline_view(view: &LenientNonceBaselineView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_rate + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_nonce_baseline("code-484", "title-214");
        let view = to_lenient_nonce_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_nonce_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
