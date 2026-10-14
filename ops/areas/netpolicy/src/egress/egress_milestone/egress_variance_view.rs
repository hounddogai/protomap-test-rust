use super::*;

/// How screens and exports show a egress variance.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressVarianceView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub burst: String,
    pub rotation_days: String,
    pub discount_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_egress_variance_view(item: &EgressVariance) -> EgressVarianceView {
    EgressVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        fingerprint: item.fingerprint.clone(),
        burst: item.burst.to_string(),
        rotation_days: item.rotation_days.to_string(),
        discount_text: format!("{:.2}", item.discount),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_variance_views(items: &[EgressVariance]) -> Vec<EgressVarianceView> {
    let mut views: Vec<EgressVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_variance_view(item));
    }
    views
}

pub fn sum_egress_variance_view(view: &EgressVarianceView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_rotation_days as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_variance("code-830", "title-413");
        let view = to_egress_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
