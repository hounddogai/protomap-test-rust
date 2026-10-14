use super::*;

use acme_kit::text;

/// How screens and exports show a release template.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseTemplateView {
    pub code: String,
    pub label_text: String,
    pub ceiling: String,
    pub ttl_seconds_text: String,
    pub audience: String,
    pub visible_text: String,
    pub grace_days: String,
    pub cluster: String,
    pub archived_text: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_release_template_view(item: &ReleaseTemplate) -> ReleaseTemplateView {
    ReleaseTemplateView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        ceiling: format!("{:.2}", item.ceiling),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        audience: item.audience.clone(),
        visible_text: format!("{}", item.visible),
        grace_days: item.grace_days.to_string(),
        cluster: item.cluster.clone(),
        archived_text: format!("{}", item.archived),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_template_views(items: &[ReleaseTemplate]) -> Vec<ReleaseTemplateView> {
    let mut views: Vec<ReleaseTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_template_view(item));
    }
    views
}

pub fn sum_release_template_view(view: &ReleaseTemplateView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ceiling + (parsed_ttl_seconds as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_template("code-401", "label-34");
        let view = to_release_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
