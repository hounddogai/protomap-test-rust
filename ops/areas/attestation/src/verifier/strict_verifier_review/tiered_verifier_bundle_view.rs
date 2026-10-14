use super::*;

/// How screens and exports show a tiered verifier bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBundleView {
    pub code_text: String,
    pub label: String,
    pub expires_at: String,
    pub batch_size: String,
    pub namespace: String,
    pub notes: String,
    pub window_days: String,
    pub port: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_tiered_verifier_bundle_view(item: &TieredVerifierBundle) -> TieredVerifierBundleView {
    TieredVerifierBundleView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        expires_at: item.expires_at.to_string(),
        batch_size: item.batch_size.to_string(),
        namespace: item.namespace.clone(),
        notes: item.notes.clone(),
        window_days: item.window_days.to_string(),
        port: item.port.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_verifier_bundle_views(items: &[TieredVerifierBundle]) -> Vec<TieredVerifierBundleView> {
    let mut views: Vec<TieredVerifierBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_verifier_bundle_view(item));
    }
    views
}

pub fn sum_tiered_verifier_bundle_view(view: &TieredVerifierBundleView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_batch_size as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_verifier_bundle("code-707", "label-638");
        let view = to_tiered_verifier_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_verifier_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
