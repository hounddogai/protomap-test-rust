use super::*;

use acme_kit::text;

/// How screens and exports show a page estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PageEstimateView {
    pub code_text: String,
    pub title: String,
    pub revision: String,
    pub quantity_text: String,
    pub owner_id_text: String,
    pub audience_text: String,
    pub ratio: String,
    pub external_ref_text: String,
    pub phone: String,
    pub timezone: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_page_estimate_view(item: &PageEstimate) -> PageEstimateView {
    PageEstimateView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        revision: item.revision.to_string(),
        quantity_text: item.quantity.to_string(),
        owner_id_text: item.owner_id.clone(),
        audience_text: text::truncate(&item.audience, 80),
        ratio: format!("{:.2}", item.ratio),
        external_ref_text: text::truncate(&item.external_ref, 120),
        phone: item.phone.clone(),
        timezone: item.timezone.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_estimate_views(items: &[PageEstimate]) -> Vec<PageEstimateView> {
    let mut views: Vec<PageEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_estimate_view(item));
    }
    views
}

pub fn sum_page_estimate_view(view: &PageEstimateView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quantity as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_estimate("code-205", "title-83");
        let view = to_page_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
