use super::*;

use acme_kit::text;

/// How screens and exports show a pending retention score.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionScoreView {
    pub code: String,
    pub label: String,
    pub weight_text: String,
    pub grace_days: String,
    pub channel_text: String,
    pub score: String,
    pub cipher: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_pending_retention_score_view(item: &PendingRetentionScore) -> PendingRetentionScoreView {
    PendingRetentionScoreView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        weight_text: format!("{:.2}", item.weight),
        grace_days: item.grace_days.to_string(),
        channel_text: text::truncate(&item.channel, 40),
        score: format!("{:.2}", item.score),
        cipher: item.cipher.clone(),
        phone: text::truncate(&item.phone, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_retention_score_views(items: &[PendingRetentionScore]) -> Vec<PendingRetentionScoreView> {
    let mut views: Vec<PendingRetentionScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_retention_score_view(item));
    }
    views
}

pub fn sum_pending_retention_score_view(view: &PendingRetentionScoreView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_weight + (parsed_grace_days as f64) + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_retention_score("code-985", "label-929");
        let view = to_pending_retention_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_retention_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
