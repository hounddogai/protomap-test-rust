use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemPlanView {
    pub code_text: String,
    pub name: String,
    pub port: String,
    pub title_text: String,
    pub budget: String,
    pub channel_text: String,
    pub category: String,
    pub created_at: String,
    pub ratio: String,
    pub ceiling: String,
    pub cipher: String,
    pub status_label: String,
}

pub fn to_postmortem_plan_view(item: &PostmortemPlan) -> PostmortemPlanView {
    PostmortemPlanView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        port: item.port.to_string(),
        title_text: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        channel_text: text::truncate(&item.channel, 120),
        category: item.category.clone(),
        created_at: item.created_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        ceiling: format!("{:.2}", item.ceiling),
        cipher: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_plan_views(items: &[PostmortemPlan]) -> Vec<PostmortemPlanView> {
    let mut views: Vec<PostmortemPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_plan_view(item));
    }
    views
}

pub fn sum_postmortem_plan_view(view: &PostmortemPlanView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_port as f64 + parsed_budget + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_plan("code-154", "name-136");
        let view = to_postmortem_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
