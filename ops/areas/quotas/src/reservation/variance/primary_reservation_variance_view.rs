use super::*;

/// How screens and exports show a primary reservation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationVarianceView {
    pub code: String,
    pub title: String,
    pub limit: String,
    pub revision_text: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_primary_reservation_variance_view(item: &PrimaryReservationVariance) -> PrimaryReservationVarianceView {
    PrimaryReservationVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        limit: item.limit.to_string(),
        revision_text: item.revision.to_string(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_reservation_variance_views(
    items: &[PrimaryReservationVariance],
) -> Vec<PrimaryReservationVarianceView> {
    let mut views: Vec<PrimaryReservationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_reservation_variance_view(item));
    }
    views
}

pub fn sum_primary_reservation_variance_view(view: &PrimaryReservationVarianceView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_reservation_variance("code-248", "title-379");
        let view = to_primary_reservation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_reservation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
