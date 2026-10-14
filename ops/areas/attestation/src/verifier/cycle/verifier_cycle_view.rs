use super::*;

use acme_kit::text;

/// How screens and exports show a verifier cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub audience: String,
    pub description_text: String,
    pub source: String,
    pub pinned: String,
    pub duration_minutes: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_verifier_cycle_view(item: &VerifierCycle) -> VerifierCycleView {
    VerifierCycleView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        summary: item.summary.clone(),
        audience: text::truncate(&item.audience, 40),
        description_text: item.description.clone(),
        source: text::truncate(&item.source, 40),
        pinned: format!("{}", item.pinned),
        duration_minutes: item.duration_minutes.to_string(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_cycle_views(items: &[VerifierCycle]) -> Vec<VerifierCycleView> {
    let mut views: Vec<VerifierCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_cycle_view(item));
    }
    views
}

pub fn sum_verifier_cycle_view(view: &VerifierCycleView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_cycle("code-209", "label-335");
        let view = to_verifier_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
