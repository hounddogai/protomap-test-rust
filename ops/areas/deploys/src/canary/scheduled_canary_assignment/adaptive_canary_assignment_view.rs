use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive canary assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryAssignmentView {
    pub code: String,
    pub label: String,
    pub country: String,
    pub checksum: String,
    pub key_id: String,
    pub channel_text: String,
    pub visible_text: String,
    pub currency: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_adaptive_canary_assignment_view(item: &AdaptiveCanaryAssignment) -> AdaptiveCanaryAssignmentView {
    AdaptiveCanaryAssignmentView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        country: item.country.clone(),
        checksum: item.checksum.clone(),
        key_id: text::truncate(&item.key_id, 120),
        channel_text: item.channel.clone(),
        visible_text: format!("{}", item.visible),
        currency: item.currency.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_canary_assignment_views(items: &[AdaptiveCanaryAssignment]) -> Vec<AdaptiveCanaryAssignmentView> {
    let mut views: Vec<AdaptiveCanaryAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_canary_assignment_view(item));
    }
    views
}

pub fn sum_adaptive_canary_assignment_view(view: &AdaptiveCanaryAssignmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_canary_assignment("code-574", "label-170");
        let view = to_adaptive_canary_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_canary_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
