use super::*;

use acme_kit::text;

/// How screens and exports show a regional usage template.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTemplateView {
    pub code: String,
    pub title_text: String,
    pub batch_size: String,
    pub host: String,
    pub expires_at: String,
    pub ratio: String,
    pub contact_email: String,
    pub rate_text: String,
    pub attempts: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_regional_usage_template_view(item: &RegionalUsageTemplate) -> RegionalUsageTemplateView {
    RegionalUsageTemplateView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 80),
        batch_size: item.batch_size.to_string(),
        host: item.host.clone(),
        expires_at: item.expires_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        contact_email: text::truncate(&item.contact_email, 40),
        rate_text: format!("{:.2}", item.rate),
        attempts: item.attempts.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_usage_template_views(items: &[RegionalUsageTemplate]) -> Vec<RegionalUsageTemplateView> {
    let mut views: Vec<RegionalUsageTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_usage_template_view(item));
    }
    views
}

pub fn sum_regional_usage_template_view(view: &RegionalUsageTemplateView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_expires_at as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_usage_template("code-72", "title-253");
        let view = to_regional_usage_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_usage_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
