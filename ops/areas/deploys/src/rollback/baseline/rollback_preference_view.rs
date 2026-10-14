use super::*;

use acme_kit::text;

/// How screens and exports show a rollback preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPreferenceView {
    pub code_text: String,
    pub label: String,
    pub source: String,
    pub expires_at: String,
    pub notes: String,
    pub visible_text: String,
    pub ratio: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_rollback_preference_view(item: &RollbackPreference) -> RollbackPreferenceView {
    RollbackPreferenceView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        source: text::truncate(&item.source, 64),
        expires_at: item.expires_at.to_string(),
        notes: item.notes.clone(),
        visible_text: format!("{}", item.visible),
        ratio: format!("{:.2}", item.ratio),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_preference_views(items: &[RollbackPreference]) -> Vec<RollbackPreferenceView> {
    let mut views: Vec<RollbackPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_preference_view(item));
    }
    views
}

pub fn sum_rollback_preference_view(view: &RollbackPreferenceView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_preference("code-938", "label-949");
        let view = to_rollback_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
