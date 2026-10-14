use super::*;

use acme_kit::text;

/// How screens and exports show a primary identity score.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityScoreView {
    pub code_text: String,
    pub title_text: String,
    pub duration_minutes: String,
    pub batch_size: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_primary_identity_score_view(item: &PrimaryIdentityScore) -> PrimaryIdentityScoreView {
    PrimaryIdentityScoreView {
        code_text: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 64),
        duration_minutes: item.duration_minutes.to_string(),
        batch_size: item.batch_size.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_identity_score_views(items: &[PrimaryIdentityScore]) -> Vec<PrimaryIdentityScoreView> {
    let mut views: Vec<PrimaryIdentityScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_identity_score_view(item));
    }
    views
}

pub fn sum_primary_identity_score_view(view: &PrimaryIdentityScoreView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_batch_size as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_identity_score("code-703", "title-644");
        let view = to_primary_identity_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_identity_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
