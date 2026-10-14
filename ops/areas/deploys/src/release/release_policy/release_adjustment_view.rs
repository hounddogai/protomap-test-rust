use super::*;

use acme_kit::text;

/// How screens and exports show a release adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAdjustmentView {
    pub code_text: String,
    pub title: String,
    pub score: String,
    pub reviewer: String,
    pub country: String,
    pub owner_id: String,
    pub source: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_release_adjustment_view(item: &ReleaseAdjustment) -> ReleaseAdjustmentView {
    ReleaseAdjustmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        score: format!("{:.2}", item.score),
        reviewer: item.reviewer.clone(),
        country: text::truncate(&item.country, 120),
        owner_id: item.owner_id.clone(),
        source: item.source.clone(),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_adjustment_views(items: &[ReleaseAdjustment]) -> Vec<ReleaseAdjustmentView> {
    let mut views: Vec<ReleaseAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_adjustment_view(item));
    }
    views
}

pub fn sum_release_adjustment_view(view: &ReleaseAdjustmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_score + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_release_adjustment("code-601", "title-667");
        let view = to_release_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_release_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
