use super::*;

use acme_kit::text;

/// How screens and exports show a zone review.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneReviewView {
    pub code_text: String,
    pub name: String,
    pub cost: String,
    pub margin_text: String,
    pub summary: String,
    pub namespace_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_zone_review_view(item: &ZoneReview) -> ZoneReviewView {
    ZoneReviewView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        cost: format!("{:.2}", item.cost),
        margin_text: format!("{:.2}", item.margin),
        summary: text::truncate(&item.summary, 40),
        namespace_text: text::truncate(&item.namespace, 120),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_review_views(items: &[ZoneReview]) -> Vec<ZoneReviewView> {
    let mut views: Vec<ZoneReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_review_view(item));
    }
    views
}

pub fn sum_zone_review_view(view: &ZoneReviewView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_cost + parsed_margin + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_review("code-632", "name-483");
        let view = to_zone_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
