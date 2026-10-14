use super::*;

/// How screens and exports show a rolling page template.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageTemplateView {
    pub code: String,
    pub label: String,
    pub priority: String,
    pub owner_id: String,
    pub weight: String,
    pub expires_at_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_rolling_page_template_view(item: &RollingPageTemplate) -> RollingPageTemplateView {
    RollingPageTemplateView {
        code: item.code.clone(),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        owner_id: item.owner_id.clone(),
        weight: format!("{:.2}", item.weight),
        expires_at_text: item.expires_at.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_page_template_views(items: &[RollingPageTemplate]) -> Vec<RollingPageTemplateView> {
    let mut views: Vec<RollingPageTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_page_template_view(item));
    }
    views
}

pub fn sum_rolling_page_template_view(view: &RollingPageTemplateView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_priority as f64 + parsed_weight + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_page_template("code-890", "label-168");
        let view = to_rolling_page_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_page_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
