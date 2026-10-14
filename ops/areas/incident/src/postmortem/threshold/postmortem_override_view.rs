use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem override.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemOverrideView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub limit_text: String,
    pub timezone: String,
    pub revision_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_postmortem_override_view(item: &PostmortemOverride) -> PostmortemOverrideView {
    PostmortemOverrideView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        currency: item.currency.clone(),
        limit_text: item.limit.to_string(),
        timezone: item.timezone.clone(),
        revision_text: item.revision.to_string(),
        cluster: text::truncate(&item.cluster, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_override_views(items: &[PostmortemOverride]) -> Vec<PostmortemOverrideView> {
    let mut views: Vec<PostmortemOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_override_view(item));
    }
    views
}

pub fn sum_postmortem_override_view(view: &PostmortemOverrideView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_override("code-361", "label-759");
        let view = to_postmortem_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
