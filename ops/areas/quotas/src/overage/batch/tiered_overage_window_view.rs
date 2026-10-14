use super::*;

use acme_kit::text;

/// How screens and exports show a tiered overage window.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageWindowView {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub rotation_days: String,
    pub key_id_text: String,
    pub source: String,
    pub limit: String,
    pub owner_id: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_tiered_overage_window_view(item: &TieredOverageWindow) -> TieredOverageWindowView {
    TieredOverageWindowView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        tenant_id: item.tenant_id.clone(),
        rotation_days: item.rotation_days.to_string(),
        key_id_text: item.key_id.clone(),
        source: item.source.clone(),
        limit: item.limit.to_string(),
        owner_id: text::truncate(&item.owner_id, 80),
        checksum_text: text::truncate(&item.checksum, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_overage_window_views(items: &[TieredOverageWindow]) -> Vec<TieredOverageWindowView> {
    let mut views: Vec<TieredOverageWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_overage_window_view(item));
    }
    views
}

pub fn sum_tiered_overage_window_view(view: &TieredOverageWindowView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_overage_window("code-657", "title-822");
        let view = to_tiered_overage_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_overage_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
