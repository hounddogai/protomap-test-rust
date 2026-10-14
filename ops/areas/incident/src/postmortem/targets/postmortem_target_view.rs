use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem target.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemTargetView {
    pub code: String,
    pub label_text: String,
    pub retries: String,
    pub score_text: String,
    pub category: String,
    pub grace_days_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_postmortem_target_view(item: &PostmortemTarget) -> PostmortemTargetView {
    PostmortemTargetView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        retries: item.retries.to_string(),
        score_text: format!("{:.2}", item.score),
        category: item.category.clone(),
        grace_days_text: item.grace_days.to_string(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_target_views(items: &[PostmortemTarget]) -> Vec<PostmortemTargetView> {
    let mut views: Vec<PostmortemTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_target_view(item));
    }
    views
}

pub fn sum_postmortem_target_view(view: &PostmortemTargetView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_retries as f64 + parsed_score + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_postmortem_target("code-697", "label-878");
        let view = to_postmortem_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_postmortem_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
