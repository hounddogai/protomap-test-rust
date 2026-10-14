use super::*;

use acme_kit::text;

/// How screens and exports show a failover notice.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverNoticeView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub title: String,
    pub audience_text: String,
    pub visible: String,
    pub key_id: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_failover_notice_view(item: &FailoverNotice) -> FailoverNoticeView {
    FailoverNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        title: text::truncate(&item.title, 120),
        audience_text: item.audience.clone(),
        visible: format!("{}", item.visible),
        key_id: text::truncate(&item.key_id, 80),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_notice_views(items: &[FailoverNotice]) -> Vec<FailoverNoticeView> {
    let mut views: Vec<FailoverNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_notice_view(item));
    }
    views
}

pub fn sum_failover_notice_view(view: &FailoverNoticeView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_notice("code-935", "label-376");
        let view = to_failover_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
