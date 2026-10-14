use super::*;

use acme_kit::text;

/// How screens and exports show a tenant template.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantTemplateView {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub cluster: String,
    pub score: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_tenant_template_view(item: &TenantTemplate) -> TenantTemplateView {
    TenantTemplateView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        channel: text::truncate(&item.channel, 40),
        cluster: text::truncate(&item.cluster, 64),
        score: format!("{:.2}", item.score),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_template_views(items: &[TenantTemplate]) -> Vec<TenantTemplateView> {
    let mut views: Vec<TenantTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_template_view(item));
    }
    views
}

pub fn sum_tenant_template_view(view: &TenantTemplateView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_score + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_template("code-516", "label-117");
        let view = to_tenant_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
