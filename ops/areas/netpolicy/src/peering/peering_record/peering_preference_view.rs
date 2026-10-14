use super::*;

use acme_kit::text;

/// How screens and exports show a peering preference.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPreferenceView {
    pub code: String,
    pub name_text: String,
    pub port: String,
    pub ceiling: String,
    pub notes_text: String,
    pub timezone: String,
    pub external_ref: String,
    pub summary_text: String,
    pub burst: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_peering_preference_view(item: &PeeringPreference) -> PeeringPreferenceView {
    PeeringPreferenceView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 64),
        port: item.port.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        notes_text: item.notes.clone(),
        timezone: text::truncate(&item.timezone, 64),
        external_ref: item.external_ref.clone(),
        summary_text: text::truncate(&item.summary, 120),
        burst: item.burst.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_preference_views(items: &[PeeringPreference]) -> Vec<PeeringPreferenceView> {
    let mut views: Vec<PeeringPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_preference_view(item));
    }
    views
}

pub fn sum_peering_preference_view(view: &PeeringPreferenceView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_port as f64 + parsed_ceiling + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_preference("code-132", "name-950");
        let view = to_peering_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
