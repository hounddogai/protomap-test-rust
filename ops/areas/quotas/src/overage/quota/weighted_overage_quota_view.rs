use super::*;

use acme_kit::text;

/// How screens and exports show a weighted overage quota.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageQuotaView {
    pub code: String,
    pub label: String,
    pub minimum: String,
    pub host: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_weighted_overage_quota_view(item: &WeightedOverageQuota) -> WeightedOverageQuotaView {
    WeightedOverageQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        minimum: format!("{:.2}", item.minimum),
        host: text::truncate(&item.host, 64),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_overage_quota_views(items: &[WeightedOverageQuota]) -> Vec<WeightedOverageQuotaView> {
    let mut views: Vec<WeightedOverageQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_overage_quota_view(item));
    }
    views
}

pub fn sum_weighted_overage_quota_view(view: &WeightedOverageQuotaView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_overage_quota("code-614", "label-502");
        let view = to_weighted_overage_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_overage_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
