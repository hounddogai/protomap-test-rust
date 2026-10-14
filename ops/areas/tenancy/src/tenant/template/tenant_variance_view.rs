use super::*;

/// How screens and exports show a tenant variance.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantVarianceView {
    pub code_text: String,
    pub label: String,
    pub minimum_text: String,
    pub issuer: String,
    pub port_text: String,
    pub reviewer: String,
    pub approved: String,
    pub host: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_tenant_variance_view(item: &TenantVariance) -> TenantVarianceView {
    TenantVarianceView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        issuer: item.issuer.clone(),
        port_text: item.port.to_string(),
        reviewer: item.reviewer.clone(),
        approved: format!("{}", item.approved),
        host: item.host.clone(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_variance_views(items: &[TenantVariance]) -> Vec<TenantVarianceView> {
    let mut views: Vec<TenantVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_variance_view(item));
    }
    views
}

pub fn sum_tenant_variance_view(view: &TenantVarianceView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_minimum + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_variance("code-37", "label-549");
        let view = to_tenant_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
