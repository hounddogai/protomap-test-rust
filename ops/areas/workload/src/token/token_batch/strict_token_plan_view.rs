use super::*;

use acme_kit::text;

/// How screens and exports show a strict token plan.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTokenPlanView {
    pub code: String,
    pub label: String,
    pub updated_at_text: String,
    pub port: String,
    pub timezone: String,
    pub rate: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_strict_token_plan_view(item: &StrictTokenPlan) -> StrictTokenPlanView {
    StrictTokenPlanView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        updated_at_text: item.updated_at.to_string(),
        port: item.port.to_string(),
        timezone: item.timezone.clone(),
        rate: format!("{:.2}", item.rate),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_token_plan_views(items: &[StrictTokenPlan]) -> Vec<StrictTokenPlanView> {
    let mut views: Vec<StrictTokenPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_token_plan_view(item));
    }
    views
}

pub fn sum_strict_token_plan_view(view: &StrictTokenPlanView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_port as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_token_plan("code-591", "label-599");
        let view = to_strict_token_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_token_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
