use super::*;

use acme_kit::text;

/// How screens and exports show a silence plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlanView {
    pub code_text: String,
    pub title: String,
    pub segment_text: String,
    pub priority: String,
    pub ttl_seconds: String,
    pub floor_value_text: String,
    pub minimum_text: String,
    pub cipher: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_silence_plan_view(item: &SilencePlan) -> SilencePlanView {
    SilencePlanView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        segment_text: text::truncate(&item.segment, 40),
        priority: item.priority.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        minimum_text: format!("{:.2}", item.minimum),
        cipher: text::truncate(&item.cipher, 64),
        currency: text::truncate(&item.currency, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_plan_views(items: &[SilencePlan]) -> Vec<SilencePlanView> {
    let mut views: Vec<SilencePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_plan_view(item));
    }
    views
}

pub fn sum_silence_plan_view(view: &SilencePlanView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_ttl_seconds as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_plan("code-89", "title-254");
        let view = to_silence_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
