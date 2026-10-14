use super::*;

use acme_kit::text;

/// How screens and exports show a page template.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTemplateView {
    pub code_text: String,
    pub label_text: String,
    pub cluster: String,
    pub archived: String,
    pub score_text: String,
    pub quota_units: String,
    pub issuer_text: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_page_template_view(item: &PageTemplate) -> PageTemplateView {
    PageTemplateView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        cluster: item.cluster.clone(),
        archived: format!("{}", item.archived),
        score_text: format!("{:.2}", item.score),
        quota_units: item.quota_units.to_string(),
        issuer_text: text::truncate(&item.issuer, 40),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_template_views(items: &[PageTemplate]) -> Vec<PageTemplateView> {
    let mut views: Vec<PageTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_template_view(item));
    }
    views
}

pub fn sum_page_template_view(view: &PageTemplateView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_score + (parsed_quota_units as f64) + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_template("code-125", "label-737");
        let view = to_page_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
