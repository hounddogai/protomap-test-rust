use super::*;

use acme_kit::text;

/// How screens and exports show a fallback data export rule.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportRuleView {
    pub code_text: String,
    pub title: String,
    pub timezone_text: String,
    pub score_text: String,
    pub maximum_text: String,
    pub description: String,
    pub attempts: String,
    pub channel_text: String,
    pub locale_text: String,
    pub ceiling_text: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_fallback_data_export_rule_view(item: &FallbackDataExportRule) -> FallbackDataExportRuleView {
    FallbackDataExportRuleView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        timezone_text: text::truncate(&item.timezone, 80),
        score_text: format!("{:.2}", item.score),
        maximum_text: format!("{:.2}", item.maximum),
        description: item.description.clone(),
        attempts: item.attempts.to_string(),
        channel_text: item.channel.clone(),
        locale_text: item.locale.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_data_export_rule_views(items: &[FallbackDataExportRule]) -> Vec<FallbackDataExportRuleView> {
    let mut views: Vec<FallbackDataExportRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_data_export_rule_view(item));
    }
    views
}

pub fn sum_fallback_data_export_rule_view(view: &FallbackDataExportRuleView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_score + parsed_maximum + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_data_export_rule("code-52", "title-389");
        let view = to_fallback_data_export_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_data_export_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
