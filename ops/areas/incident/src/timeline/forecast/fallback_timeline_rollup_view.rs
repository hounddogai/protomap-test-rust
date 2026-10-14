use super::*;

/// How screens and exports show a fallback timeline rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTimelineRollupView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub maximum: String,
    pub namespace: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_fallback_timeline_rollup_view(item: &FallbackTimelineRollup) -> FallbackTimelineRollupView {
    FallbackTimelineRollupView {
        code: item.code.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        maximum: format!("{:.2}", item.maximum),
        namespace: item.namespace.clone(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_timeline_rollup_views(items: &[FallbackTimelineRollup]) -> Vec<FallbackTimelineRollupView> {
    let mut views: Vec<FallbackTimelineRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_timeline_rollup_view(item));
    }
    views
}

pub fn sum_fallback_timeline_rollup_view(view: &FallbackTimelineRollupView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_discount + parsed_maximum + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_timeline_rollup("code-623", "label-803");
        let view = to_fallback_timeline_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_timeline_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
