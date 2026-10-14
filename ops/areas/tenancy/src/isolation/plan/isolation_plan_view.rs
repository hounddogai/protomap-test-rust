use super::*;

use acme_kit::text;

/// How screens and exports show a isolation plan.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationPlanView {
    pub code: String,
    pub title_text: String,
    pub active_text: String,
    pub visible: String,
    pub discount: String,
    pub name_text: String,
    pub source_text: String,
    pub segment_text: String,
    pub status_label: String,
}

pub fn to_isolation_plan_view(item: &IsolationPlan) -> IsolationPlanView {
    IsolationPlanView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 80),
        active_text: format!("{}", item.active),
        visible: format!("{}", item.visible),
        discount: format!("{:.2}", item.discount),
        name_text: item.name.clone(),
        source_text: item.source.clone(),
        segment_text: text::truncate(&item.segment, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_plan_views(items: &[IsolationPlan]) -> Vec<IsolationPlanView> {
    let mut views: Vec<IsolationPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_plan_view(item));
    }
    views
}

pub fn sum_isolation_plan_view(view: &IsolationPlanView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_plan("code-28", "title-537");
        let view = to_isolation_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
