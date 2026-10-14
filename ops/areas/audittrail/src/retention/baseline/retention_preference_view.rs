use super::*;

use acme_kit::text;

/// How screens and exports show a retention preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPreferenceView {
    pub code_text: String,
    pub title: String,
    pub reviewer: String,
    pub currency: String,
    pub audience: String,
    pub score: String,
    pub active: String,
    pub ratio_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_retention_preference_view(item: &RetentionPreference) -> RetentionPreferenceView {
    RetentionPreferenceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        reviewer: text::truncate(&item.reviewer, 40),
        currency: item.currency.clone(),
        audience: item.audience.clone(),
        score: format!("{:.2}", item.score),
        active: format!("{}", item.active),
        ratio_text: format!("{:.2}", item.ratio),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_preference_views(items: &[RetentionPreference]) -> Vec<RetentionPreferenceView> {
    let mut views: Vec<RetentionPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_preference_view(item));
    }
    views
}

pub fn sum_retention_preference_view(view: &RetentionPreferenceView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_score + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_preference("code-172", "title-953");
        let view = to_retention_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
