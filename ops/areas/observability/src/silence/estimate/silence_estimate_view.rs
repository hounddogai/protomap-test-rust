use super::*;

/// How screens and exports show a silence estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEstimateView {
    pub code_text: String,
    pub label: String,
    pub name: String,
    pub external_ref: String,
    pub scope: String,
    pub score_text: String,
    pub locale: String,
    pub starts_at: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_silence_estimate_view(item: &SilenceEstimate) -> SilenceEstimateView {
    SilenceEstimateView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        name: item.name.clone(),
        external_ref: item.external_ref.clone(),
        scope: item.scope.clone(),
        score_text: format!("{:.2}", item.score),
        locale: item.locale.clone(),
        starts_at: item.starts_at.to_string(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_estimate_views(items: &[SilenceEstimate]) -> Vec<SilenceEstimateView> {
    let mut views: Vec<SilenceEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_estimate_view(item));
    }
    views
}

pub fn sum_silence_estimate_view(view: &SilenceEstimateView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_score + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_silence_estimate("code-644", "label-767");
        let view = to_silence_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_silence_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
