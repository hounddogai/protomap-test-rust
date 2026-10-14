use super::*;

use acme_kit::text;

/// How screens and exports show a manual allowlist exception.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistExceptionView {
    pub code: String,
    pub title: String,
    pub version_text: String,
    pub rotation_days: String,
    pub unit: String,
    pub reviewer_text: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_manual_allowlist_exception_view(item: &ManualAllowlistException) -> ManualAllowlistExceptionView {
    ManualAllowlistExceptionView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        version_text: item.version.to_string(),
        rotation_days: item.rotation_days.to_string(),
        unit: text::truncate(&item.unit, 64),
        reviewer_text: item.reviewer.clone(),
        cipher_text: text::truncate(&item.cipher, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_allowlist_exception_views(items: &[ManualAllowlistException]) -> Vec<ManualAllowlistExceptionView> {
    let mut views: Vec<ManualAllowlistExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_allowlist_exception_view(item));
    }
    views
}

pub fn sum_manual_allowlist_exception_view(view: &ManualAllowlistExceptionView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_version as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_allowlist_exception("code-452", "title-508");
        let view = to_manual_allowlist_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_allowlist_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
