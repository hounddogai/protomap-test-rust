use super::*;

use acme_kit::text;

/// How screens and exports show a lenient policy variance.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyVarianceView {
    pub code: String,
    pub title: String,
    pub sequence: String,
    pub phone: String,
    pub summary: String,
    pub label: String,
    pub quota_units: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_lenient_policy_variance_view(item: &LenientPolicyVariance) -> LenientPolicyVarianceView {
    LenientPolicyVarianceView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        sequence: item.sequence.to_string(),
        phone: item.phone.clone(),
        summary: item.summary.clone(),
        label: text::truncate(&item.label, 80),
        quota_units: item.quota_units.to_string(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_policy_variance_views(items: &[LenientPolicyVariance]) -> Vec<LenientPolicyVarianceView> {
    let mut views: Vec<LenientPolicyVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_policy_variance_view(item));
    }
    views
}

pub fn sum_lenient_policy_variance_view(view: &LenientPolicyVarianceView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_quota_units as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_policy_variance("code-632", "title-255");
        let view = to_lenient_policy_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_policy_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
