use super::*;

/// How screens and exports show a seasonal canary threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryThresholdView {
    pub code: String,
    pub name_text: String,
    pub priority: String,
    pub quantity_text: String,
    pub ceiling: String,
    pub channel: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_seasonal_canary_threshold_view(item: &SeasonalCanaryThreshold) -> SeasonalCanaryThresholdView {
    SeasonalCanaryThresholdView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        priority: item.priority.to_string(),
        quantity_text: item.quantity.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        channel: item.channel.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_canary_threshold_views(items: &[SeasonalCanaryThreshold]) -> Vec<SeasonalCanaryThresholdView> {
    let mut views: Vec<SeasonalCanaryThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_canary_threshold_view(item));
    }
    views
}

pub fn sum_seasonal_canary_threshold_view(view: &SeasonalCanaryThresholdView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_quantity as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_canary_threshold("code-326", "name-636");
        let view = to_seasonal_canary_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_canary_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
