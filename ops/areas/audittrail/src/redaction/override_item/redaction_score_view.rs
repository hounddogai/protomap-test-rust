use super::*;

use acme_kit::text;

/// How screens and exports show a redaction score.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScoreView {
    pub code_text: String,
    pub name: String,
    pub owner_id_text: String,
    pub quota_units: String,
    pub unit: String,
    pub ratio_text: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_redaction_score_view(item: &RedactionScore) -> RedactionScoreView {
    RedactionScoreView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        owner_id_text: item.owner_id.clone(),
        quota_units: item.quota_units.to_string(),
        unit: text::truncate(&item.unit, 64),
        ratio_text: format!("{:.2}", item.ratio),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_score_views(items: &[RedactionScore]) -> Vec<RedactionScoreView> {
    let mut views: Vec<RedactionScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_score_view(item));
    }
    views
}

pub fn sum_redaction_score_view(view: &RedactionScoreView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_redaction_score("code-100", "name-964");
        let view = to_redaction_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_redaction_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
