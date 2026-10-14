use super::*;

/// How screens and exports show a postmortem rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRollupView {
    pub code_text: String,
    pub label: String,
    pub duration_minutes: String,
    pub source_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_postmortem_rollup_view(item: &PostmortemRollup) -> PostmortemRollupView {
    PostmortemRollupView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        source_text: item.source.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_rollup_views(items: &[PostmortemRollup]) -> Vec<PostmortemRollupView> {
    let mut views: Vec<PostmortemRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_rollup_view(item));
    }
    views
}

pub fn sum_postmortem_rollup_view(view: &PostmortemRollupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_rollup("code-172", "label-453");
        let view = to_postmortem_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
