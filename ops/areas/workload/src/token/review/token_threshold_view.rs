use super::*;

use acme_kit::text;

/// How screens and exports show a token threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenThresholdView {
    pub code_text: String,
    pub label_text: String,
    pub threshold: String,
    pub checksum_text: String,
    pub duration_minutes: String,
    pub score_text: String,
    pub floor_value_text: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_token_threshold_view(item: &TokenThreshold) -> TokenThresholdView {
    TokenThresholdView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        threshold: format!("{:.2}", item.threshold),
        checksum_text: text::truncate(&item.checksum, 40),
        duration_minutes: item.duration_minutes.to_string(),
        score_text: format!("{:.2}", item.score),
        floor_value_text: format!("{:.2}", item.floor_value),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_threshold_views(items: &[TokenThreshold]) -> Vec<TokenThresholdView> {
    let mut views: Vec<TokenThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_threshold_view(item));
    }
    views
}

pub fn sum_token_threshold_view(view: &TokenThresholdView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_threshold + (parsed_duration_minutes as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_token_threshold("code-706", "label-905");
        let view = to_token_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_token_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
