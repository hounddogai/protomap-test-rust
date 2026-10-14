use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive isolation template.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationTemplateView {
    pub code_text: String,
    pub label: String,
    pub region: String,
    pub approved: String,
    pub external_ref_text: String,
    pub duration_minutes_text: String,
    pub principal: String,
    pub owner_id_text: String,
    pub rate: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_adaptive_isolation_template_view(item: &AdaptiveIsolationTemplate) -> AdaptiveIsolationTemplateView {
    AdaptiveIsolationTemplateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        region: item.region.clone(),
        approved: format!("{}", item.approved),
        external_ref_text: text::truncate(&item.external_ref, 120),
        duration_minutes_text: item.duration_minutes.to_string(),
        principal: item.principal.clone(),
        owner_id_text: text::truncate(&item.owner_id, 120),
        rate: format!("{:.2}", item.rate),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_isolation_template_views(items: &[AdaptiveIsolationTemplate]) -> Vec<AdaptiveIsolationTemplateView> {
    let mut views: Vec<AdaptiveIsolationTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_isolation_template_view(item));
    }
    views
}

pub fn sum_adaptive_isolation_template_view(view: &AdaptiveIsolationTemplateView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_isolation_template("code-787", "label-941");
        let view = to_adaptive_isolation_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_isolation_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
