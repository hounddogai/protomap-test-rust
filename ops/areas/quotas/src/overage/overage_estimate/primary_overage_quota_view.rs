use super::*;

use acme_kit::text;

/// How screens and exports show a primary overage quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageQuotaView {
    pub code_text: String,
    pub label: String,
    pub discount: String,
    pub source: String,
    pub floor_value: String,
    pub external_ref: String,
    pub score: String,
    pub audience: String,
    pub timezone_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_primary_overage_quota_view(item: &PrimaryOverageQuota) -> PrimaryOverageQuotaView {
    PrimaryOverageQuotaView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        discount: format!("{:.2}", item.discount),
        source: text::truncate(&item.source, 120),
        floor_value: format!("{:.2}", item.floor_value),
        external_ref: text::truncate(&item.external_ref, 120),
        score: format!("{:.2}", item.score),
        audience: text::truncate(&item.audience, 64),
        timezone_text: item.timezone.clone(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_overage_quota_views(items: &[PrimaryOverageQuota]) -> Vec<PrimaryOverageQuotaView> {
    let mut views: Vec<PrimaryOverageQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_overage_quota_view(item));
    }
    views
}

pub fn sum_primary_overage_quota_view(view: &PrimaryOverageQuotaView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_discount + parsed_floor_value + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_overage_quota("code-447", "label-345");
        let view = to_primary_overage_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_overage_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
