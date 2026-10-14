use super::*;

/// How screens and exports show a tiered timeline notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredTimelineNoticeView {
    pub code_text: String,
    pub label: String,
    pub score: String,
    pub threshold_text: String,
    pub owner_id_text: String,
    pub weight_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_tiered_timeline_notice_view(item: &TieredTimelineNotice) -> TieredTimelineNoticeView {
    TieredTimelineNoticeView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        threshold_text: format!("{:.2}", item.threshold),
        owner_id_text: item.owner_id.clone(),
        weight_text: format!("{:.2}", item.weight),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_timeline_notice_views(items: &[TieredTimelineNotice]) -> Vec<TieredTimelineNoticeView> {
    let mut views: Vec<TieredTimelineNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_timeline_notice_view(item));
    }
    views
}

pub fn sum_tiered_timeline_notice_view(view: &TieredTimelineNoticeView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_score + parsed_threshold + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_timeline_notice("code-269", "label-813");
        let view = to_tiered_timeline_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_timeline_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
