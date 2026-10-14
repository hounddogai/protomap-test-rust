use super::*;

/// How screens and exports show a zone claim.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneClaimView {
    pub code: String,
    pub title_text: String,
    pub country: String,
    pub capacity: String,
    pub budget: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_zone_claim_view(item: &ZoneClaim) -> ZoneClaimView {
    ZoneClaimView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        country: item.country.clone(),
        capacity: item.capacity.to_string(),
        budget: format!("{:.2}", item.budget),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_zone_claim_views(items: &[ZoneClaim]) -> Vec<ZoneClaimView> {
    let mut views: Vec<ZoneClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_zone_claim_view(item));
    }
    views
}

pub fn sum_zone_claim_view(view: &ZoneClaimView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_zone_claim("code-101", "title-317");
        let view = to_zone_claim_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_zone_claim_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
