use super::*;

/// How screens and exports show a legacy evidence tier.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceTierView {
    pub code: String,
    pub label: String,
    pub minimum_text: String,
    pub rotation_days_text: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_legacy_evidence_tier_view(item: &LegacyEvidenceTier) -> LegacyEvidenceTierView {
    LegacyEvidenceTierView {
        code: item.code.clone(),
        label: item.label.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        rotation_days_text: item.rotation_days.to_string(),
        notes_text: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_evidence_tier_views(items: &[LegacyEvidenceTier]) -> Vec<LegacyEvidenceTierView> {
    let mut views: Vec<LegacyEvidenceTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_evidence_tier_view(item));
    }
    views
}

pub fn sum_legacy_evidence_tier_view(view: &LegacyEvidenceTierView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_minimum + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_evidence_tier("code-342", "label-973");
        let view = to_legacy_evidence_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_evidence_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
