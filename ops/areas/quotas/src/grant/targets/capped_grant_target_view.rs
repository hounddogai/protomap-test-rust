use super::*;

use acme_kit::text;

/// How screens and exports show a capped grant target.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantTargetView {
    pub code_text: String,
    pub name: String,
    pub cluster: String,
    pub maximum: String,
    pub channel_text: String,
    pub currency: String,
    pub ratio: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_capped_grant_target_view(item: &CappedGrantTarget) -> CappedGrantTargetView {
    CappedGrantTargetView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        cluster: item.cluster.clone(),
        maximum: format!("{:.2}", item.maximum),
        channel_text: text::truncate(&item.channel, 40),
        currency: text::truncate(&item.currency, 64),
        ratio: format!("{:.2}", item.ratio),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_grant_target_views(items: &[CappedGrantTarget]) -> Vec<CappedGrantTargetView> {
    let mut views: Vec<CappedGrantTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_grant_target_view(item));
    }
    views
}

pub fn sum_capped_grant_target_view(view: &CappedGrantTargetView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_maximum + parsed_ratio + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_grant_target("code-994", "name-258");
        let view = to_capped_grant_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_grant_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
