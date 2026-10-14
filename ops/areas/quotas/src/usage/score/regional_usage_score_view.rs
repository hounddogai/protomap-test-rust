use super::*;

use acme_kit::text;

/// How screens and exports show a regional usage score.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageScoreView {
    pub code: String,
    pub title: String,
    pub ratio_text: String,
    pub notes_text: String,
    pub discount: String,
    pub limit_text: String,
    pub weight_text: String,
    pub segment_text: String,
    pub revision: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_regional_usage_score_view(item: &RegionalUsageScore) -> RegionalUsageScoreView {
    RegionalUsageScoreView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        ratio_text: format!("{:.2}", item.ratio),
        notes_text: item.notes.clone(),
        discount: format!("{:.2}", item.discount),
        limit_text: item.limit.to_string(),
        weight_text: format!("{:.2}", item.weight),
        segment_text: item.segment.clone(),
        revision: item.revision.to_string(),
        timezone: text::truncate(&item.timezone, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_usage_score_views(items: &[RegionalUsageScore]) -> Vec<RegionalUsageScoreView> {
    let mut views: Vec<RegionalUsageScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_usage_score_view(item));
    }
    views
}

pub fn sum_regional_usage_score_view(view: &RegionalUsageScoreView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_ratio + parsed_discount + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_usage_score("code-298", "title-335");
        let view = to_regional_usage_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_usage_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
