use super::*;

use acme_kit::text;

/// How screens and exports show a token quota.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenQuotaView {
    pub code: String,
    pub name_text: String,
    pub priority: String,
    pub key_id_text: String,
    pub external_ref: String,
    pub ceiling: String,
    pub floor_value: String,
    pub budget: String,
    pub pinned: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_token_quota_view(item: &TokenQuota) -> TokenQuotaView {
    TokenQuotaView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        priority: item.priority.to_string(),
        key_id_text: item.key_id.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        ceiling: format!("{:.2}", item.ceiling),
        floor_value: format!("{:.2}", item.floor_value),
        budget: format!("{:.2}", item.budget),
        pinned: format!("{}", item.pinned),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_quota_views(items: &[TokenQuota]) -> Vec<TokenQuotaView> {
    let mut views: Vec<TokenQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_quota_view(item));
    }
    views
}

pub fn sum_token_quota_view(view: &TokenQuotaView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_priority as f64 + parsed_ceiling + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_quota("code-286", "name-336");
        let view = to_token_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
