use super::*;

/// How screens and exports show a batched silence segment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceSegmentView {
    pub code: String,
    pub title: String,
    pub cost_text: String,
    pub weight_text: String,
    pub duration_minutes: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_batched_silence_segment_view(item: &BatchedSilenceSegment) -> BatchedSilenceSegmentView {
    BatchedSilenceSegmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        cost_text: format!("{:.2}", item.cost),
        weight_text: format!("{:.2}", item.weight),
        duration_minutes: item.duration_minutes.to_string(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_silence_segment_views(items: &[BatchedSilenceSegment]) -> Vec<BatchedSilenceSegmentView> {
    let mut views: Vec<BatchedSilenceSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_silence_segment_view(item));
    }
    views
}

pub fn sum_batched_silence_segment_view(view: &BatchedSilenceSegmentView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_cost + parsed_weight + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_silence_segment("code-51", "title-587");
        let view = to_batched_silence_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_silence_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
