use super::*;

use acme_kit::text;

/// How screens and exports show a probe cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCycleView {
    pub code: String,
    pub name: String,
    pub country_text: String,
    pub duration_minutes: String,
    pub ttl_seconds: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_probe_cycle_view(item: &ProbeCycle) -> ProbeCycleView {
    ProbeCycleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        country_text: item.country.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_cycle_views(items: &[ProbeCycle]) -> Vec<ProbeCycleView> {
    let mut views: Vec<ProbeCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_cycle_view(item));
    }
    views
}

pub fn sum_probe_cycle_view(view: &ProbeCycleView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_ttl_seconds as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_cycle("code-639", "name-704");
        let view = to_probe_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
