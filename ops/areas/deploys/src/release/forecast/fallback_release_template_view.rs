use super::*;

use acme_kit::text;

/// How screens and exports show a fallback release template.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseTemplateView {
    pub code_text: String,
    pub label: String,
    pub namespace: String,
    pub ends_at: String,
    pub description: String,
    pub active: String,
    pub cost: String,
    pub retries: String,
    pub score_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_fallback_release_template_view(item: &FallbackReleaseTemplate) -> FallbackReleaseTemplateView {
    FallbackReleaseTemplateView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 40),
        namespace: item.namespace.clone(),
        ends_at: item.ends_at.to_string(),
        description: text::truncate(&item.description, 80),
        active: format!("{}", item.active),
        cost: format!("{:.2}", item.cost),
        retries: item.retries.to_string(),
        score_text: format!("{:.2}", item.score),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_release_template_views(items: &[FallbackReleaseTemplate]) -> Vec<FallbackReleaseTemplateView> {
    let mut views: Vec<FallbackReleaseTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_release_template_view(item));
    }
    views
}

pub fn sum_fallback_release_template_view(view: &FallbackReleaseTemplateView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_cost + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_release_template("code-502", "label-961");
        let view = to_fallback_release_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_release_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
