use super::*;

use acme_kit::text;

/// How screens and exports show a tiered query notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryNoticeView {
    pub code: String,
    pub title: String,
    pub pinned: String,
    pub unit: String,
    pub sequence: String,
    pub description: String,
    pub minimum: String,
    pub grace_days: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_tiered_query_notice_view(item: &TieredQueryNotice) -> TieredQueryNoticeView {
    TieredQueryNoticeView {
        code: item.code.clone(),
        title: item.title.clone(),
        pinned: format!("{}", item.pinned),
        unit: item.unit.clone(),
        sequence: item.sequence.to_string(),
        description: text::truncate(&item.description, 40),
        minimum: format!("{:.2}", item.minimum),
        grace_days: item.grace_days.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_query_notice_views(items: &[TieredQueryNotice]) -> Vec<TieredQueryNoticeView> {
    let mut views: Vec<TieredQueryNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_query_notice_view(item));
    }
    views
}

pub fn sum_tiered_query_notice_view(view: &TieredQueryNoticeView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_minimum + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_query_notice("code-892", "title-326");
        let view = to_tiered_query_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_query_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
