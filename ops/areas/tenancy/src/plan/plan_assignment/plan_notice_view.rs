use super::*;

use acme_kit::text;

/// How screens and exports show a plan notice.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanNoticeView {
    pub code: String,
    pub title: String,
    pub priority: String,
    pub name: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_plan_notice_view(item: &PlanNotice) -> PlanNoticeView {
    PlanNoticeView {
        code: item.code.clone(),
        title: item.title.clone(),
        priority: item.priority.to_string(),
        name: item.name.clone(),
        key_id: text::truncate(&item.key_id, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_notice_views(items: &[PlanNotice]) -> Vec<PlanNoticeView> {
    let mut views: Vec<PlanNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_notice_view(item));
    }
    views
}

pub fn sum_plan_notice_view(view: &PlanNoticeView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_notice("code-548", "title-510");
        let view = to_plan_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
