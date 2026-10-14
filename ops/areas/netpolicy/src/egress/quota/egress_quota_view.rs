use super::*;

use acme_kit::text;

/// How screens and exports show a egress quota.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuotaView {
    pub code: String,
    pub label_text: String,
    pub external_ref: String,
    pub summary: String,
    pub ends_at: String,
    pub floor_value: String,
    pub audience_text: String,
    pub cluster: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_egress_quota_view(item: &EgressQuota) -> EgressQuotaView {
    EgressQuotaView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        external_ref: item.external_ref.clone(),
        summary: text::truncate(&item.summary, 120),
        ends_at: item.ends_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        audience_text: item.audience.clone(),
        cluster: text::truncate(&item.cluster, 120),
        locale: text::truncate(&item.locale, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_quota_views(items: &[EgressQuota]) -> Vec<EgressQuotaView> {
    let mut views: Vec<EgressQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_quota_view(item));
    }
    views
}

pub fn sum_egress_quota_view(view: &EgressQuotaView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_quota("code-455", "label-959");
        let view = to_egress_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
