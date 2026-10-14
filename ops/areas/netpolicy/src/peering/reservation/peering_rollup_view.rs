use super::*;

use acme_kit::text;

/// How screens and exports show a peering rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRollupView {
    pub code: String,
    pub name_text: String,
    pub expires_at_text: String,
    pub score: String,
    pub source: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_peering_rollup_view(item: &PeeringRollup) -> PeeringRollupView {
    PeeringRollupView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        expires_at_text: item.expires_at.to_string(),
        score: format!("{:.2}", item.score),
        source: item.source.clone(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_rollup_views(items: &[PeeringRollup]) -> Vec<PeeringRollupView> {
    let mut views: Vec<PeeringRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_rollup_view(item));
    }
    views
}

pub fn sum_peering_rollup_view(view: &PeeringRollupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_rollup("code-531", "name-753");
        let view = to_peering_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
