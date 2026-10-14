use super::*;

use acme_kit::text;

/// How screens and exports show a federation quota.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationQuotaView {
    pub code: String,
    pub name_text: String,
    pub duration_minutes: String,
    pub visible: String,
    pub weight_text: String,
    pub capacity: String,
    pub ceiling: String,
    pub approved: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_federation_quota_view(item: &FederationQuota) -> FederationQuotaView {
    FederationQuotaView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        duration_minutes: item.duration_minutes.to_string(),
        visible: format!("{}", item.visible),
        weight_text: format!("{:.2}", item.weight),
        capacity: item.capacity.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        approved: format!("{}", item.approved),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_quota_views(items: &[FederationQuota]) -> Vec<FederationQuotaView> {
    let mut views: Vec<FederationQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_quota_view(item));
    }
    views
}

pub fn sum_federation_quota_view(view: &FederationQuotaView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_weight + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_quota("code-586", "name-382");
        let view = to_federation_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
