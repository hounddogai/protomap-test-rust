use super::*;

/// How screens and exports show a adaptive grant template.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplateView {
    pub code: String,
    pub title: String,
    pub discount_text: String,
    pub reviewer_text: String,
    pub source_text: String,
    pub visible: String,
    pub ends_at: String,
    pub version: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_adaptive_grant_template_view(item: &AdaptiveGrantTemplate) -> AdaptiveGrantTemplateView {
    AdaptiveGrantTemplateView {
        code: item.code.clone(),
        title: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        reviewer_text: item.reviewer.clone(),
        source_text: item.source.clone(),
        visible: format!("{}", item.visible),
        ends_at: item.ends_at.to_string(),
        version: item.version.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_grant_template_views(items: &[AdaptiveGrantTemplate]) -> Vec<AdaptiveGrantTemplateView> {
    let mut views: Vec<AdaptiveGrantTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_grant_template_view(item));
    }
    views
}

pub fn sum_adaptive_grant_template_view(view: &AdaptiveGrantTemplateView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_discount + (parsed_ends_at as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_grant_template("code-16", "title-636");
        let view = to_adaptive_grant_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_grant_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
