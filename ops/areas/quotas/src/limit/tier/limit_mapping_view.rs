use super::*;

use acme_kit::text;

/// How screens and exports show a limit mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitMappingView {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub sequence_text: String,
    pub window_days: String,
    pub approved: String,
    pub budget: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_limit_mapping_view(item: &LimitMapping) -> LimitMappingView {
    LimitMappingView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        channel: item.channel.clone(),
        sequence_text: item.sequence.to_string(),
        window_days: item.window_days.to_string(),
        approved: format!("{}", item.approved),
        budget: format!("{:.2}", item.budget),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_mapping_views(items: &[LimitMapping]) -> Vec<LimitMappingView> {
    let mut views: Vec<LimitMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_mapping_view(item));
    }
    views
}

pub fn sum_limit_mapping_view(view: &LimitMappingView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_window_days as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_mapping("code-455", "label-549");
        let view = to_limit_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
