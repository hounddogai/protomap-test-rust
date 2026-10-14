use super::*;

use acme_kit::text;

/// How screens and exports show a canary target.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTargetView {
    pub code: String,
    pub title: String,
    pub expires_at: String,
    pub summary: String,
    pub name: String,
    pub visible_text: String,
    pub attempts: String,
    pub checksum_text: String,
    pub label: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_canary_target_view(item: &CanaryTarget) -> CanaryTargetView {
    CanaryTargetView {
        code: item.code.clone(),
        title: item.title.clone(),
        expires_at: item.expires_at.to_string(),
        summary: text::truncate(&item.summary, 40),
        name: text::truncate(&item.name, 80),
        visible_text: format!("{}", item.visible),
        attempts: item.attempts.to_string(),
        checksum_text: text::truncate(&item.checksum, 40),
        label: item.label.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_target_views(items: &[CanaryTarget]) -> Vec<CanaryTargetView> {
    let mut views: Vec<CanaryTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_target_view(item));
    }
    views
}

pub fn sum_canary_target_view(view: &CanaryTargetView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_attempts as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_target("code-329", "title-103");
        let view = to_canary_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
