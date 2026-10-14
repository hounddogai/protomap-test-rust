use super::*;

use acme_kit::text;

/// How screens and exports show a peering score.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringScoreView {
    pub code: String,
    pub label: String,
    pub name: String,
    pub rate_text: String,
    pub port: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_peering_score_view(item: &PeeringScore) -> PeeringScoreView {
    PeeringScoreView {
        code: item.code.clone(),
        label: item.label.clone(),
        name: item.name.clone(),
        rate_text: format!("{:.2}", item.rate),
        port: item.port.to_string(),
        description: text::truncate(&item.description, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_score_views(items: &[PeeringScore]) -> Vec<PeeringScoreView> {
    let mut views: Vec<PeeringScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_score_view(item));
    }
    views
}

pub fn sum_peering_score_view(view: &PeeringScoreView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_rate + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_score("code-536", "label-899");
        let view = to_peering_score_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_score_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
