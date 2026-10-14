use super::*;

/// How screens and exports show a tiered probe agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeAgreementView {
    pub code: String,
    pub name_text: String,
    pub cluster: String,
    pub priority: String,
    pub duration_minutes_text: String,
    pub archived: String,
    pub notes: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_tiered_probe_agreement_view(item: &TieredProbeAgreement) -> TieredProbeAgreementView {
    TieredProbeAgreementView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        cluster: item.cluster.clone(),
        priority: item.priority.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        archived: format!("{}", item.archived),
        notes: item.notes.clone(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_probe_agreement_views(items: &[TieredProbeAgreement]) -> Vec<TieredProbeAgreementView> {
    let mut views: Vec<TieredProbeAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_probe_agreement_view(item));
    }
    views
}

pub fn sum_tiered_probe_agreement_view(view: &TieredProbeAgreementView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_probe_agreement("code-594", "name-854");
        let view = to_tiered_probe_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_probe_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
