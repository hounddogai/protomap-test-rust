use super::*;

use acme_kit::text;

/// How screens and exports show a timeline plan.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlanView {
    pub code: String,
    pub name: String,
    pub margin: String,
    pub minimum_text: String,
    pub owner_id: String,
    pub key_id: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_timeline_plan_view(item: &TimelinePlan) -> TimelinePlanView {
    TimelinePlanView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        minimum_text: format!("{:.2}", item.minimum),
        owner_id: item.owner_id.clone(),
        key_id: text::truncate(&item.key_id, 80),
        region: text::truncate(&item.region, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_plan_views(items: &[TimelinePlan]) -> Vec<TimelinePlanView> {
    let mut views: Vec<TimelinePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_plan_view(item));
    }
    views
}

pub fn sum_timeline_plan_view(view: &TimelinePlanView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_margin + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_timeline_plan("code-189", "name-854");
        let view = to_timeline_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_timeline_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
