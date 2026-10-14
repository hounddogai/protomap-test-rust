use super::*;

use acme_kit::text;

/// How screens and exports show a sink milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkMilestoneView {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub checksum_text: String,
    pub phone_text: String,
    pub cipher: String,
    pub created_at: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_sink_milestone_view(item: &SinkMilestone) -> SinkMilestoneView {
    SinkMilestoneView {
        code: item.code.clone(),
        label: item.label.clone(),
        timezone: item.timezone.clone(),
        checksum_text: item.checksum.clone(),
        phone_text: text::truncate(&item.phone, 120),
        cipher: item.cipher.clone(),
        created_at: item.created_at.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_milestone_views(items: &[SinkMilestone]) -> Vec<SinkMilestoneView> {
    let mut views: Vec<SinkMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_milestone_view(item));
    }
    views
}

pub fn sum_sink_milestone_view(view: &SinkMilestoneView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_sink_milestone("code-947", "label-654");
        let view = to_sink_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_sink_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
