use super::*;

use acme_kit::text;

/// How screens and exports show a batched retention tier.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionTierView {
    pub code: String,
    pub name_text: String,
    pub quota_units: String,
    pub threshold: String,
    pub summary: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_batched_retention_tier_view(item: &BatchedRetentionTier) -> BatchedRetentionTierView {
    BatchedRetentionTierView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        threshold: format!("{:.2}", item.threshold),
        summary: text::truncate(&item.summary, 40),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_retention_tier_views(items: &[BatchedRetentionTier]) -> Vec<BatchedRetentionTierView> {
    let mut views: Vec<BatchedRetentionTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_retention_tier_view(item));
    }
    views
}

pub fn sum_batched_retention_tier_view(view: &BatchedRetentionTierView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_retention_tier("code-854", "name-142");
        let view = to_batched_retention_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_retention_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
