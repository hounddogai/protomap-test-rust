use super::*;

/// How screens and exports show a adaptive forecast cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastCycleView {
    pub code: String,
    pub title_text: String,
    pub limit_text: String,
    pub discount: String,
    pub sequence: String,
    pub host: String,
    pub created_at: String,
    pub notes: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_adaptive_forecast_cycle_view(item: &AdaptiveForecastCycle) -> AdaptiveForecastCycleView {
    AdaptiveForecastCycleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        limit_text: item.limit.to_string(),
        discount: format!("{:.2}", item.discount),
        sequence: item.sequence.to_string(),
        host: item.host.clone(),
        created_at: item.created_at.to_string(),
        notes: item.notes.clone(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_forecast_cycle_views(items: &[AdaptiveForecastCycle]) -> Vec<AdaptiveForecastCycleView> {
    let mut views: Vec<AdaptiveForecastCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_forecast_cycle_view(item));
    }
    views
}

pub fn sum_adaptive_forecast_cycle_view(view: &AdaptiveForecastCycleView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_limit as f64 + parsed_discount + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_forecast_cycle("code-962", "title-158");
        let view = to_adaptive_forecast_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_forecast_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
