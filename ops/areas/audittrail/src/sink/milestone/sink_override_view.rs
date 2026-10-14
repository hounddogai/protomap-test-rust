use super::*;

use acme_kit::text;

/// How screens and exports show a sink override.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverrideView {
    pub code_text: String,
    pub name_text: String,
    pub description: String,
    pub cost: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_sink_override_view(item: &SinkOverride) -> SinkOverrideView {
    SinkOverrideView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        description: item.description.clone(),
        cost: format!("{:.2}", item.cost),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_override_views(items: &[SinkOverride]) -> Vec<SinkOverrideView> {
    let mut views: Vec<SinkOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_override_view(item));
    }
    views
}

pub fn sum_sink_override_view(view: &SinkOverrideView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_cost + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_override("code-415", "name-979");
        let view = to_sink_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
