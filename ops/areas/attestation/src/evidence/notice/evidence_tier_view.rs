use super::*;

use acme_kit::text;

/// How screens and exports show a evidence tier.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTierView {
    pub code: String,
    pub label: String,
    pub weight: String,
    pub pinned: String,
    pub phone: String,
    pub approved: String,
    pub rate: String,
    pub visible: String,
    pub priority_text: String,
    pub quantity: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_evidence_tier_view(item: &EvidenceTier) -> EvidenceTierView {
    EvidenceTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        weight: format!("{:.2}", item.weight),
        pinned: format!("{}", item.pinned),
        phone: text::truncate(&item.phone, 64),
        approved: format!("{}", item.approved),
        rate: format!("{:.2}", item.rate),
        visible: format!("{}", item.visible),
        priority_text: item.priority.to_string(),
        quantity: item.quantity.to_string(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_tier_views(items: &[EvidenceTier]) -> Vec<EvidenceTierView> {
    let mut views: Vec<EvidenceTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_tier_view(item));
    }
    views
}

pub fn sum_evidence_tier_view(view: &EvidenceTierView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_weight + parsed_rate + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_evidence_tier("code-576", "label-287");
        let view = to_evidence_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_evidence_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
