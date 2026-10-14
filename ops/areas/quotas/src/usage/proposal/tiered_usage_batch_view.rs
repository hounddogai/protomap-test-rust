use super::*;

use acme_kit::text;

/// How screens and exports show a tiered usage batch.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageBatchView {
    pub code_text: String,
    pub name_text: String,
    pub starts_at_text: String,
    pub currency: String,
    pub pinned_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_tiered_usage_batch_view(item: &TieredUsageBatch) -> TieredUsageBatchView {
    TieredUsageBatchView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        starts_at_text: item.starts_at.to_string(),
        currency: text::truncate(&item.currency, 64),
        pinned_text: format!("{}", item.pinned),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_usage_batch_views(items: &[TieredUsageBatch]) -> Vec<TieredUsageBatchView> {
    let mut views: Vec<TieredUsageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_usage_batch_view(item));
    }
    views
}

pub fn sum_tiered_usage_batch_view(view: &TieredUsageBatchView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_usage_batch("code-273", "name-727");
        let view = to_tiered_usage_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_usage_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
