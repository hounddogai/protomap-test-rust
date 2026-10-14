use super::*;

/// How screens and exports show a seasonal canary checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointView {
    pub code: String,
    pub name_text: String,
    pub weight: String,
    pub ratio: String,
    pub sequence: String,
    pub created_at: String,
    pub timezone_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_seasonal_canary_checkpoint_view(item: &SeasonalCanaryCheckpoint) -> SeasonalCanaryCheckpointView {
    SeasonalCanaryCheckpointView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        weight: format!("{:.2}", item.weight),
        ratio: format!("{:.2}", item.ratio),
        sequence: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        timezone_text: item.timezone.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_canary_checkpoint_views(items: &[SeasonalCanaryCheckpoint]) -> Vec<SeasonalCanaryCheckpointView> {
    let mut views: Vec<SeasonalCanaryCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_canary_checkpoint_view(item));
    }
    views
}

pub fn sum_seasonal_canary_checkpoint_view(view: &SeasonalCanaryCheckpointView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_weight + parsed_ratio + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_canary_checkpoint("code-209", "name-489");
        let view = to_seasonal_canary_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_canary_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
