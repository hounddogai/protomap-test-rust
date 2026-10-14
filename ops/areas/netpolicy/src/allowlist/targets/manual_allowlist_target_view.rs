use super::*;

use acme_kit::text;

/// How screens and exports show a manual allowlist target.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistTargetView {
    pub code_text: String,
    pub title: String,
    pub phone: String,
    pub active: String,
    pub window_days: String,
    pub issuer_text: String,
    pub status_label: String,
}

pub fn to_manual_allowlist_target_view(item: &ManualAllowlistTarget) -> ManualAllowlistTargetView {
    ManualAllowlistTargetView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        phone: text::truncate(&item.phone, 120),
        active: format!("{}", item.active),
        window_days: item.window_days.to_string(),
        issuer_text: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_allowlist_target_views(items: &[ManualAllowlistTarget]) -> Vec<ManualAllowlistTargetView> {
    let mut views: Vec<ManualAllowlistTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_allowlist_target_view(item));
    }
    views
}

pub fn sum_manual_allowlist_target_view(view: &ManualAllowlistTargetView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_allowlist_target("code-586", "title-985");
        let view = to_manual_allowlist_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_allowlist_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
