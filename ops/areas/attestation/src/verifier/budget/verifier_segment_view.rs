use super::*;

use acme_kit::text;

/// How screens and exports show a verifier segment.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSegmentView {
    pub code: String,
    pub name_text: String,
    pub ends_at: String,
    pub quantity_text: String,
    pub score: String,
    pub burst: String,
    pub capacity_text: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_verifier_segment_view(item: &VerifierSegment) -> VerifierSegmentView {
    VerifierSegmentView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        quantity_text: item.quantity.to_string(),
        score: format!("{:.2}", item.score),
        burst: item.burst.to_string(),
        capacity_text: item.capacity.to_string(),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_segment_views(items: &[VerifierSegment]) -> Vec<VerifierSegmentView> {
    let mut views: Vec<VerifierSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_segment_view(item));
    }
    views
}

pub fn sum_verifier_segment_view(view: &VerifierSegmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_quantity as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_segment("code-616", "name-371");
        let view = to_verifier_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
