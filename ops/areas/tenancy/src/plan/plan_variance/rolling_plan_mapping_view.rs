use super::*;

use acme_kit::text;

/// How screens and exports show a rolling plan mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanMappingView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub timezone_text: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_rolling_plan_mapping_view(item: &RollingPlanMapping) -> RollingPlanMappingView {
    RollingPlanMappingView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        cipher: item.cipher.clone(),
        timezone_text: item.timezone.clone(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_plan_mapping_views(items: &[RollingPlanMapping]) -> Vec<RollingPlanMappingView> {
    let mut views: Vec<RollingPlanMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_plan_mapping_view(item));
    }
    views
}

pub fn sum_rolling_plan_mapping_view(view: &RollingPlanMappingView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_plan_mapping("code-360", "title-341");
        let view = to_rolling_plan_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_plan_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
