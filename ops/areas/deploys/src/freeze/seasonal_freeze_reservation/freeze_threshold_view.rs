use super::*;

use acme_kit::text;

/// How screens and exports show a freeze threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeThresholdView {
    pub code_text: String,
    pub label: String,
    pub title: String,
    pub currency_text: String,
    pub cipher: String,
    pub budget: String,
    pub ttl_seconds_text: String,
    pub retries: String,
    pub pinned: String,
    pub sequence_text: String,
    pub status_label: String,
}

pub fn to_freeze_threshold_view(item: &FreezeThreshold) -> FreezeThresholdView {
    FreezeThresholdView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        title: item.title.clone(),
        currency_text: text::truncate(&item.currency, 120),
        cipher: item.cipher.clone(),
        budget: format!("{:.2}", item.budget),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        retries: item.retries.to_string(),
        pinned: format!("{}", item.pinned),
        sequence_text: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_threshold_views(items: &[FreezeThreshold]) -> Vec<FreezeThresholdView> {
    let mut views: Vec<FreezeThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_threshold_view(item));
    }
    views
}

pub fn sum_freeze_threshold_view(view: &FreezeThresholdView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_budget + (parsed_ttl_seconds as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_threshold("code-507", "label-324");
        let view = to_freeze_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
