use super::*;

/// How screens and exports show a adaptive freeze segment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeSegmentView {
    pub code: String,
    pub name_text: String,
    pub checksum_text: String,
    pub contact_email_text: String,
    pub burst: String,
    pub visible: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_adaptive_freeze_segment_view(item: &AdaptiveFreezeSegment) -> AdaptiveFreezeSegmentView {
    AdaptiveFreezeSegmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        checksum_text: item.checksum.clone(),
        contact_email_text: item.contact_email.clone(),
        burst: item.burst.to_string(),
        visible: format!("{}", item.visible),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_freeze_segment_views(items: &[AdaptiveFreezeSegment]) -> Vec<AdaptiveFreezeSegmentView> {
    let mut views: Vec<AdaptiveFreezeSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_freeze_segment_view(item));
    }
    views
}

pub fn sum_adaptive_freeze_segment_view(view: &AdaptiveFreezeSegmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_freeze_segment("code-589", "name-761");
        let view = to_adaptive_freeze_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_freeze_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
