use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled segment item.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentItemView {
    pub code_text: String,
    pub name: String,
    pub issuer_text: String,
    pub created_at_text: String,
    pub principal: String,
    pub host: String,
    pub fingerprint: String,
    pub archived: String,
    pub margin: String,
    pub summary_text: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_scheduled_segment_item_view(item: &ScheduledSegmentItem) -> ScheduledSegmentItemView {
    ScheduledSegmentItemView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        created_at_text: item.created_at.to_string(),
        principal: item.principal.clone(),
        host: item.host.clone(),
        fingerprint: item.fingerprint.clone(),
        archived: format!("{}", item.archived),
        margin: format!("{:.2}", item.margin),
        summary_text: item.summary.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_segment_item_views(items: &[ScheduledSegmentItem]) -> Vec<ScheduledSegmentItemView> {
    let mut views: Vec<ScheduledSegmentItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_segment_item_view(item));
    }
    views
}

pub fn sum_scheduled_segment_item_view(view: &ScheduledSegmentItemView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_margin + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_segment_item("code-277", "name-520");
        let view = to_scheduled_segment_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_segment_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
