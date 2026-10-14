use super::*;

use acme_kit::text;

/// How screens and exports show a overage template.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageTemplateView {
    pub code: String,
    pub name: String,
    pub region_text: String,
    pub tenant_id: String,
    pub maximum: String,
    pub source: String,
    pub active: String,
    pub priority: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_overage_template_view(item: &OverageTemplate) -> OverageTemplateView {
    OverageTemplateView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        region_text: text::truncate(&item.region, 80),
        tenant_id: item.tenant_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        source: item.source.clone(),
        active: format!("{}", item.active),
        priority: item.priority.to_string(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_template_views(items: &[OverageTemplate]) -> Vec<OverageTemplateView> {
    let mut views: Vec<OverageTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_template_view(item));
    }
    views
}

pub fn sum_overage_template_view(view: &OverageTemplateView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_maximum + (parsed_priority as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_template("code-95", "name-352");
        let view = to_overage_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
