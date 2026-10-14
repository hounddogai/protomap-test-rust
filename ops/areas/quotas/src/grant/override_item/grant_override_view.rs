use super::*;

use acme_kit::text;

/// How screens and exports show a grant override.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantOverrideView {
    pub code_text: String,
    pub label: String,
    pub region_text: String,
    pub capacity: String,
    pub minimum: String,
    pub sequence_text: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_grant_override_view(item: &GrantOverride) -> GrantOverrideView {
    GrantOverrideView {
        code_text: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 80),
        region_text: text::truncate(&item.region, 40),
        capacity: item.capacity.to_string(),
        minimum: format!("{:.2}", item.minimum),
        sequence_text: item.sequence.to_string(),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_override_views(items: &[GrantOverride]) -> Vec<GrantOverrideView> {
    let mut views: Vec<GrantOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_override_view(item));
    }
    views
}

pub fn sum_grant_override_view(view: &GrantOverrideView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_minimum + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_override("code-961", "label-612");
        let view = to_grant_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
