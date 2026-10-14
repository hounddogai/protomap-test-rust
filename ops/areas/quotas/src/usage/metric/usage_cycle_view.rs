use super::*;

/// How screens and exports show a usage cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageCycleView {
    pub code_text: String,
    pub title: String,
    pub tenant_id: String,
    pub label: String,
    pub updated_at: String,
    pub expires_at: String,
    pub ends_at: String,
    pub rate: String,
    pub issuer: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_usage_cycle_view(item: &UsageCycle) -> UsageCycleView {
    UsageCycleView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        tenant_id: item.tenant_id.clone(),
        label: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        expires_at: item.expires_at.to_string(),
        ends_at: item.ends_at.to_string(),
        rate: format!("{:.2}", item.rate),
        issuer: item.issuer.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_cycle_views(items: &[UsageCycle]) -> Vec<UsageCycleView> {
    let mut views: Vec<UsageCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_cycle_view(item));
    }
    views
}

pub fn sum_usage_cycle_view(view: &UsageCycleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_expires_at as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_cycle("code-900", "title-394");
        let view = to_usage_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
