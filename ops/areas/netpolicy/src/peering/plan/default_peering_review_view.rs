use super::*;

/// How screens and exports show a default peering review.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewView {
    pub code: String,
    pub label: String,
    pub archived_text: String,
    pub cost: String,
    pub ttl_seconds_text: String,
    pub revision_text: String,
    pub country: String,
    pub issuer: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_default_peering_review_view(item: &DefaultPeeringReview) -> DefaultPeeringReviewView {
    DefaultPeeringReviewView {
        code: item.code.clone(),
        label: item.label.clone(),
        archived_text: format!("{}", item.archived),
        cost: format!("{:.2}", item.cost),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        revision_text: item.revision.to_string(),
        country: item.country.clone(),
        issuer: item.issuer.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_peering_review_views(items: &[DefaultPeeringReview]) -> Vec<DefaultPeeringReviewView> {
    let mut views: Vec<DefaultPeeringReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_peering_review_view(item));
    }
    views
}

pub fn sum_default_peering_review_view(view: &DefaultPeeringReviewView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_cost + (parsed_ttl_seconds as f64) + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_peering_review("code-350", "label-787");
        let view = to_default_peering_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_peering_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
