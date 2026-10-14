use super::*;

/// How screens and exports show a tiered limit estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEstimateView {
    pub code: String,
    pub label_text: String,
    pub expires_at: String,
    pub limit: String,
    pub updated_at: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_tiered_limit_estimate_view(item: &TieredLimitEstimate) -> TieredLimitEstimateView {
    TieredLimitEstimateView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        expires_at: item.expires_at.to_string(),
        limit: item.limit.to_string(),
        updated_at: item.updated_at.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_limit_estimate_views(items: &[TieredLimitEstimate]) -> Vec<TieredLimitEstimateView> {
    let mut views: Vec<TieredLimitEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_limit_estimate_view(item));
    }
    views
}

pub fn sum_tiered_limit_estimate_view(view: &TieredLimitEstimateView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_limit as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_limit_estimate("code-389", "label-881");
        let view = to_tiered_limit_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_limit_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
