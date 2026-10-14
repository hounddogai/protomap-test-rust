use super::*;

use acme_kit::text;

/// How screens and exports show a regional suspension profile.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionProfileView {
    pub code: String,
    pub title: String,
    pub approved_text: String,
    pub rotation_days: String,
    pub limit_text: String,
    pub floor_value_text: String,
    pub score: String,
    pub margin: String,
    pub attempts: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_regional_suspension_profile_view(item: &RegionalSuspensionProfile) -> RegionalSuspensionProfileView {
    RegionalSuspensionProfileView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        approved_text: format!("{}", item.approved),
        rotation_days: item.rotation_days.to_string(),
        limit_text: item.limit.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        score: format!("{:.2}", item.score),
        margin: format!("{:.2}", item.margin),
        attempts: item.attempts.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_suspension_profile_views(items: &[RegionalSuspensionProfile]) -> Vec<RegionalSuspensionProfileView> {
    let mut views: Vec<RegionalSuspensionProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_suspension_profile_view(item));
    }
    views
}

pub fn sum_regional_suspension_profile_view(view: &RegionalSuspensionProfileView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_limit as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_suspension_profile("code-887", "title-425");
        let view = to_regional_suspension_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_suspension_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
