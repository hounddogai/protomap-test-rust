use super::*;

use acme_kit::text;

/// How screens and exports show a silence cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCycleView {
    pub code_text: String,
    pub label: String,
    pub version: String,
    pub ends_at_text: String,
    pub budget: String,
    pub active: String,
    pub notes: String,
    pub margin: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_silence_cycle_view(item: &SilenceCycle) -> SilenceCycleView {
    SilenceCycleView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        version: item.version.to_string(),
        ends_at_text: item.ends_at.to_string(),
        budget: format!("{:.2}", item.budget),
        active: format!("{}", item.active),
        notes: text::truncate(&item.notes, 64),
        margin: format!("{:.2}", item.margin),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_cycle_views(items: &[SilenceCycle]) -> Vec<SilenceCycleView> {
    let mut views: Vec<SilenceCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_cycle_view(item));
    }
    views
}

pub fn sum_silence_cycle_view(view: &SilenceCycleView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_version as f64 + (parsed_ends_at as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_cycle("code-992", "label-155");
        let view = to_silence_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
