use super::*;

use acme_kit::text;

/// How screens and exports show a tenant target.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantTargetView {
    pub code: String,
    pub title: String,
    pub threshold: String,
    pub unit: String,
    pub segment: String,
    pub host: String,
    pub ratio_text: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_tenant_target_view(item: &TenantTarget) -> TenantTargetView {
    TenantTargetView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        threshold: format!("{:.2}", item.threshold),
        unit: item.unit.clone(),
        segment: item.segment.clone(),
        host: item.host.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_target_views(items: &[TenantTarget]) -> Vec<TenantTargetView> {
    let mut views: Vec<TenantTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_target_view(item));
    }
    views
}

pub fn sum_tenant_target_view(view: &TenantTargetView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_threshold + parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_target("code-575", "title-249");
        let view = to_tenant_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
