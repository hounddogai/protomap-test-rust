use super::*;

use acme_kit::text;

/// How screens and exports show a primary failover preference.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverPreferenceView {
    pub code: String,
    pub title: String,
    pub quantity: String,
    pub host_text: String,
    pub budget: String,
    pub score: String,
    pub source: String,
    pub revision_text: String,
    pub audience: String,
    pub expires_at: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_primary_failover_preference_view(item: &PrimaryFailoverPreference) -> PrimaryFailoverPreferenceView {
    PrimaryFailoverPreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        quantity: item.quantity.to_string(),
        host_text: item.host.clone(),
        budget: format!("{:.2}", item.budget),
        score: format!("{:.2}", item.score),
        source: text::truncate(&item.source, 64),
        revision_text: item.revision.to_string(),
        audience: text::truncate(&item.audience, 80),
        expires_at: item.expires_at.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_failover_preference_views(items: &[PrimaryFailoverPreference]) -> Vec<PrimaryFailoverPreferenceView> {
    let mut views: Vec<PrimaryFailoverPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_failover_preference_view(item));
    }
    views
}

pub fn sum_primary_failover_preference_view(view: &PrimaryFailoverPreferenceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_budget + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_failover_preference("code-554", "title-363");
        let view = to_primary_failover_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_failover_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
