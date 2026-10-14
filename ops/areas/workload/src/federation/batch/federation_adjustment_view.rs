use super::*;

/// How screens and exports show a federation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAdjustmentView {
    pub code: String,
    pub title: String,
    pub limit: String,
    pub created_at: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_federation_adjustment_view(item: &FederationAdjustment) -> FederationAdjustmentView {
    FederationAdjustmentView {
        code: item.code.clone(),
        title: item.title.clone(),
        limit: item.limit.to_string(),
        created_at: item.created_at.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_adjustment_views(items: &[FederationAdjustment]) -> Vec<FederationAdjustmentView> {
    let mut views: Vec<FederationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_adjustment_view(item));
    }
    views
}

pub fn sum_federation_adjustment_view(view: &FederationAdjustmentView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_adjustment("code-136", "title-142");
        let view = to_federation_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
