use super::*;

use acme_kit::text;

/// How screens and exports show a capped peering manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringManifestView {
    pub code: String,
    pub name: String,
    pub duration_minutes_text: String,
    pub amount: String,
    pub cost: String,
    pub score: String,
    pub tenant_id_text: String,
    pub discount: String,
    pub contact_email: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_capped_peering_manifest_view(item: &CappedPeeringManifest) -> CappedPeeringManifestView {
    CappedPeeringManifestView {
        code: item.code.clone(),
        name: item.name.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        amount: format!("{:.2}", item.amount),
        cost: format!("{:.2}", item.cost),
        score: format!("{:.2}", item.score),
        tenant_id_text: item.tenant_id.clone(),
        discount: format!("{:.2}", item.discount),
        contact_email: text::truncate(&item.contact_email, 40),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_peering_manifest_views(items: &[CappedPeeringManifest]) -> Vec<CappedPeeringManifestView> {
    let mut views: Vec<CappedPeeringManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_peering_manifest_view(item));
    }
    views
}

pub fn sum_capped_peering_manifest_view(view: &CappedPeeringManifestView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_amount + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_peering_manifest("code-32", "name-906");
        let view = to_capped_peering_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_peering_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
