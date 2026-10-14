use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive rollback bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackBundleView {
    pub code: String,
    pub title: String,
    pub contact_email_text: String,
    pub label: String,
    pub namespace: String,
    pub priority: String,
    pub description: String,
    pub audience: String,
    pub ratio: String,
    pub notes: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_adaptive_rollback_bundle_view(item: &AdaptiveRollbackBundle) -> AdaptiveRollbackBundleView {
    AdaptiveRollbackBundleView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 120),
        contact_email_text: item.contact_email.clone(),
        label: item.label.clone(),
        namespace: item.namespace.clone(),
        priority: item.priority.to_string(),
        description: item.description.clone(),
        audience: item.audience.clone(),
        ratio: format!("{:.2}", item.ratio),
        notes: text::truncate(&item.notes, 40),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_rollback_bundle_views(items: &[AdaptiveRollbackBundle]) -> Vec<AdaptiveRollbackBundleView> {
    let mut views: Vec<AdaptiveRollbackBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_rollback_bundle_view(item));
    }
    views
}

pub fn sum_adaptive_rollback_bundle_view(view: &AdaptiveRollbackBundleView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_priority as f64 + parsed_ratio + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_rollback_bundle("code-151", "title-365");
        let view = to_adaptive_rollback_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_rollback_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
