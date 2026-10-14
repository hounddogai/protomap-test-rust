use super::*;

/// How screens and exports show a scheduled page target.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageTargetView {
    pub code: String,
    pub name: String,
    pub visible: String,
    pub owner_id: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_page_target_view(item: &ScheduledPageTarget) -> ScheduledPageTargetView {
    ScheduledPageTargetView {
        code: item.code.clone(),
        name: item.name.clone(),
        visible: format!("{}", item.visible),
        owner_id: item.owner_id.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_page_target_views(items: &[ScheduledPageTarget]) -> Vec<ScheduledPageTargetView> {
    let mut views: Vec<ScheduledPageTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_page_target_view(item));
    }
    views
}

pub fn sum_scheduled_page_target_view(view: &ScheduledPageTargetView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_page_target("code-563", "name-28");
        let view = to_scheduled_page_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_page_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
