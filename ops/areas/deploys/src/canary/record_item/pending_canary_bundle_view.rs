use super::*;

/// How screens and exports show a pending canary bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryBundleView {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub limit: String,
    pub country: String,
    pub updated_at: String,
    pub key_id_text: String,
    pub threshold: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_pending_canary_bundle_view(item: &PendingCanaryBundle) -> PendingCanaryBundleView {
    PendingCanaryBundleView {
        code: item.code.clone(),
        label: item.label.clone(),
        locale: item.locale.clone(),
        limit: item.limit.to_string(),
        country: item.country.clone(),
        updated_at: item.updated_at.to_string(),
        key_id_text: item.key_id.clone(),
        threshold: format!("{:.2}", item.threshold),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_canary_bundle_views(items: &[PendingCanaryBundle]) -> Vec<PendingCanaryBundleView> {
    let mut views: Vec<PendingCanaryBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_canary_bundle_view(item));
    }
    views
}

pub fn sum_pending_canary_bundle_view(view: &PendingCanaryBundleView) -> Result<f64, String> {
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
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_updated_at as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_canary_bundle("code-846", "label-744");
        let view = to_pending_canary_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_canary_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
