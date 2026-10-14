use super::*;

use acme_kit::text;

/// How screens and exports show a pending query mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryMappingView {
    pub code: String,
    pub label: String,
    pub external_ref_text: String,
    pub title: String,
    pub audience: String,
    pub ends_at: String,
    pub limit: String,
    pub country_text: String,
    pub cluster: String,
    pub rate: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_pending_query_mapping_view(item: &PendingQueryMapping) -> PendingQueryMappingView {
    PendingQueryMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        external_ref_text: item.external_ref.clone(),
        title: item.title.clone(),
        audience: item.audience.clone(),
        ends_at: item.ends_at.to_string(),
        limit: item.limit.to_string(),
        country_text: item.country.clone(),
        cluster: item.cluster.clone(),
        rate: format!("{:.2}", item.rate),
        contact_email_text: text::truncate(&item.contact_email, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_query_mapping_views(items: &[PendingQueryMapping]) -> Vec<PendingQueryMappingView> {
    let mut views: Vec<PendingQueryMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_query_mapping_view(item));
    }
    views
}

pub fn sum_pending_query_mapping_view(view: &PendingQueryMappingView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_limit as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_query_mapping("code-233", "label-796");
        let view = to_pending_query_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_query_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
