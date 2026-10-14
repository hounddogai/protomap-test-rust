use super::*;

use acme_kit::text;

/// How screens and exports show a batched tenant summary.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantSummaryView {
    pub code: String,
    pub label_text: String,
    pub active_text: String,
    pub locale_text: String,
    pub approved: String,
    pub segment: String,
    pub country_text: String,
    pub cipher_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_batched_tenant_summary_view(item: &BatchedTenantSummary) -> BatchedTenantSummaryView {
    BatchedTenantSummaryView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        active_text: format!("{}", item.active),
        locale_text: item.locale.clone(),
        approved: format!("{}", item.approved),
        segment: text::truncate(&item.segment, 80),
        country_text: item.country.clone(),
        cipher_text: item.cipher.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_tenant_summary_views(items: &[BatchedTenantSummary]) -> Vec<BatchedTenantSummaryView> {
    let mut views: Vec<BatchedTenantSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_tenant_summary_view(item));
    }
    views
}

pub fn sum_batched_tenant_summary_view(view: &BatchedTenantSummaryView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_tenant_summary("code-77", "label-495");
        let view = to_batched_tenant_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_tenant_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
