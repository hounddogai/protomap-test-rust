use super::*;

use acme_kit::text;

/// How screens and exports show a suspension review.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReviewView {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub notes: String,
    pub title: String,
    pub created_at: String,
    pub phone_text: String,
    pub source: String,
    pub margin_text: String,
    pub minimum: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_suspension_review_view(item: &SuspensionReview) -> SuspensionReviewView {
    SuspensionReviewView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        issuer: text::truncate(&item.issuer, 64),
        notes: item.notes.clone(),
        title: item.title.clone(),
        created_at: item.created_at.to_string(),
        phone_text: item.phone.clone(),
        source: text::truncate(&item.source, 120),
        margin_text: format!("{:.2}", item.margin),
        minimum: format!("{:.2}", item.minimum),
        category: text::truncate(&item.category, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_review_views(items: &[SuspensionReview]) -> Vec<SuspensionReviewView> {
    let mut views: Vec<SuspensionReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_review_view(item));
    }
    views
}

pub fn sum_suspension_review_view(view: &SuspensionReviewView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_margin + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_review("code-127", "label-474");
        let view = to_suspension_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
