use super::*;

use acme_kit::text;

/// How screens and exports show a lenient canary checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryCheckpointView {
    pub code: String,
    pub label: String,
    pub tenant_id_text: String,
    pub ends_at_text: String,
    pub attempts: String,
    pub discount_text: String,
    pub limit_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_lenient_canary_checkpoint_view(item: &LenientCanaryCheckpoint) -> LenientCanaryCheckpointView {
    LenientCanaryCheckpointView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        tenant_id_text: item.tenant_id.clone(),
        ends_at_text: item.ends_at.to_string(),
        attempts: item.attempts.to_string(),
        discount_text: format!("{:.2}", item.discount),
        limit_text: item.limit.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_canary_checkpoint_views(items: &[LenientCanaryCheckpoint]) -> Vec<LenientCanaryCheckpointView> {
    let mut views: Vec<LenientCanaryCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_canary_checkpoint_view(item));
    }
    views
}

pub fn sum_lenient_canary_checkpoint_view(view: &LenientCanaryCheckpointView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_attempts as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_canary_checkpoint("code-991", "label-616");
        let view = to_lenient_canary_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_canary_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
