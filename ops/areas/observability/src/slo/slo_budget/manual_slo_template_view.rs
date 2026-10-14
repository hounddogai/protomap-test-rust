use super::*;

use acme_kit::text;

/// How screens and exports show a manual slo template.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub ends_at: String,
    pub locale_text: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_manual_slo_template_view(item: &ManualSloTemplate) -> ManualSloTemplateView {
    ManualSloTemplateView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        currency: item.currency.clone(),
        ends_at: item.ends_at.to_string(),
        locale_text: item.locale.clone(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_slo_template_views(items: &[ManualSloTemplate]) -> Vec<ManualSloTemplateView> {
    let mut views: Vec<ManualSloTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_slo_template_view(item));
    }
    views
}

pub fn sum_manual_slo_template_view(view: &ManualSloTemplateView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_slo_template("code-898", "label-833");
        let view = to_manual_slo_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_slo_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
