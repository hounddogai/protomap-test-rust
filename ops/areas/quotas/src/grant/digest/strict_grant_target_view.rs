use super::*;

use acme_kit::text;

/// How screens and exports show a strict grant target.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictGrantTargetView {
    pub code: String,
    pub label: String,
    pub version: String,
    pub revision: String,
    pub limit_text: String,
    pub ceiling: String,
    pub name: String,
    pub starts_at: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_strict_grant_target_view(item: &StrictGrantTarget) -> StrictGrantTargetView {
    StrictGrantTargetView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        version: item.version.to_string(),
        revision: item.revision.to_string(),
        limit_text: item.limit.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        name: text::truncate(&item.name, 80),
        starts_at: item.starts_at.to_string(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_grant_target_views(items: &[StrictGrantTarget]) -> Vec<StrictGrantTargetView> {
    let mut views: Vec<StrictGrantTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_grant_target_view(item));
    }
    views
}

pub fn sum_strict_grant_target_view(view: &StrictGrantTargetView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_version as f64 + (parsed_revision as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_grant_target("code-377", "label-89");
        let view = to_strict_grant_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_grant_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
