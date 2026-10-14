use super::*;

/// How screens and exports show a probe segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSegmentView {
    pub code: String,
    pub label: String,
    pub grace_days_text: String,
    pub created_at: String,
    pub expires_at_text: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_probe_segment_view(item: &ProbeSegment) -> ProbeSegmentView {
    ProbeSegmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        grace_days_text: item.grace_days.to_string(),
        created_at: item.created_at.to_string(),
        expires_at_text: item.expires_at.to_string(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_segment_views(items: &[ProbeSegment]) -> Vec<ProbeSegmentView> {
    let mut views: Vec<ProbeSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_segment_view(item));
    }
    views
}

pub fn sum_probe_segment_view(view: &ProbeSegmentView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_created_at as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_segment("code-561", "label-564");
        let view = to_probe_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
