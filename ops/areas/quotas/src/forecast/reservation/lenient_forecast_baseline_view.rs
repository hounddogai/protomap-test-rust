use super::*;

use acme_kit::text;

/// How screens and exports show a lenient forecast baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaselineView {
    pub code_text: String,
    pub title_text: String,
    pub notes_text: String,
    pub sequence: String,
    pub created_at_text: String,
    pub priority: String,
    pub ratio_text: String,
    pub capacity: String,
    pub audience: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_lenient_forecast_baseline_view(item: &LenientForecastBaseline) -> LenientForecastBaselineView {
    LenientForecastBaselineView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        notes_text: text::truncate(&item.notes, 40),
        sequence: item.sequence.to_string(),
        created_at_text: item.created_at.to_string(),
        priority: item.priority.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        capacity: item.capacity.to_string(),
        audience: item.audience.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_forecast_baseline_views(items: &[LenientForecastBaseline]) -> Vec<LenientForecastBaselineView> {
    let mut views: Vec<LenientForecastBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_forecast_baseline_view(item));
    }
    views
}

pub fn sum_lenient_forecast_baseline_view(view: &LenientForecastBaselineView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_created_at as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_forecast_baseline("code-669", "title-776");
        let view = to_lenient_forecast_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_forecast_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
