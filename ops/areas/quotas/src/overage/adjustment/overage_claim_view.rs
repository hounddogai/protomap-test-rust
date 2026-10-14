use super::*;

use acme_kit::text;

/// How screens and exports show a overage claim.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageClaimView {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub summary_text: String,
    pub fingerprint_text: String,
    pub threshold: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_overage_claim_view(item: &OverageClaim) -> OverageClaimView {
    OverageClaimView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        cluster: item.cluster.clone(),
        summary_text: item.summary.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 120),
        threshold: format!("{:.2}", item.threshold),
        unit: text::truncate(&item.unit, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_claim_views(items: &[OverageClaim]) -> Vec<OverageClaimView> {
    let mut views: Vec<OverageClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_claim_view(item));
    }
    views
}

pub fn sum_overage_claim_view(view: &OverageClaimView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_claim("code-804", "title-484");
        let view = to_overage_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
