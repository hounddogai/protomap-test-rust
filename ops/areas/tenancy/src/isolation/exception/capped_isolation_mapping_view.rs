use super::*;

use acme_kit::text;

/// How screens and exports show a capped isolation mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMappingView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub visible_text: String,
    pub contact_email: String,
    pub cost_text: String,
    pub tenant_id: String,
    pub attempts_text: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_capped_isolation_mapping_view(item: &CappedIsolationMapping) -> CappedIsolationMappingView {
    CappedIsolationMappingView {
        code: item.code.clone(),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        visible_text: format!("{}", item.visible),
        contact_email: text::truncate(&item.contact_email, 80),
        cost_text: format!("{:.2}", item.cost),
        tenant_id: text::truncate(&item.tenant_id, 120),
        attempts_text: item.attempts.to_string(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_isolation_mapping_views(items: &[CappedIsolationMapping]) -> Vec<CappedIsolationMappingView> {
    let mut views: Vec<CappedIsolationMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_isolation_mapping_view(item));
    }
    views
}

pub fn sum_capped_isolation_mapping_view(view: &CappedIsolationMappingView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_cost + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_isolation_mapping("code-855", "title-610");
        let view = to_capped_isolation_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_isolation_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
