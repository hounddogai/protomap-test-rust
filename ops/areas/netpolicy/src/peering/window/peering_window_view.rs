use super::*;

use acme_kit::text;

/// How screens and exports show a peering window.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringWindowView {
    pub code: String,
    pub title: String,
    pub limit: String,
    pub key_id: String,
    pub maximum: String,
    pub duration_minutes: String,
    pub cost: String,
    pub timezone: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_peering_window_view(item: &PeeringWindow) -> PeeringWindowView {
    PeeringWindowView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        limit: item.limit.to_string(),
        key_id: text::truncate(&item.key_id, 40),
        maximum: format!("{:.2}", item.maximum),
        duration_minutes: item.duration_minutes.to_string(),
        cost: format!("{:.2}", item.cost),
        timezone: item.timezone.clone(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_window_views(items: &[PeeringWindow]) -> Vec<PeeringWindowView> {
    let mut views: Vec<PeeringWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_window_view(item));
    }
    views
}

pub fn sum_peering_window_view(view: &PeeringWindowView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_limit as f64 + parsed_maximum + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_window("code-166", "title-938");
        let view = to_peering_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
