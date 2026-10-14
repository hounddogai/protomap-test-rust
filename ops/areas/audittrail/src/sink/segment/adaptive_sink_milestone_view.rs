use super::*;

/// How screens and exports show a adaptive sink milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSinkMilestoneView {
    pub code: String,
    pub name: String,
    pub threshold: String,
    pub cipher_text: String,
    pub duration_minutes_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_adaptive_sink_milestone_view(item: &AdaptiveSinkMilestone) -> AdaptiveSinkMilestoneView {
    AdaptiveSinkMilestoneView {
        code: item.code.clone(),
        name: item.name.clone(),
        threshold: format!("{:.2}", item.threshold),
        cipher_text: item.cipher.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_sink_milestone_views(items: &[AdaptiveSinkMilestone]) -> Vec<AdaptiveSinkMilestoneView> {
    let mut views: Vec<AdaptiveSinkMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_sink_milestone_view(item));
    }
    views
}

pub fn sum_adaptive_sink_milestone_view(view: &AdaptiveSinkMilestoneView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_threshold + (parsed_duration_minutes as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_sink_milestone("code-331", "name-952");
        let view = to_adaptive_sink_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_sink_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
