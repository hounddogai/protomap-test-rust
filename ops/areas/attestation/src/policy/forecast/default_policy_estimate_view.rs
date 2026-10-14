use super::*;

/// How screens and exports show a default policy estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyEstimateView {
    pub code: String,
    pub title: String,
    pub name: String,
    pub contact_email_text: String,
    pub sequence: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_default_policy_estimate_view(item: &DefaultPolicyEstimate) -> DefaultPolicyEstimateView {
    DefaultPolicyEstimateView {
        code: item.code.clone(),
        title: item.title.clone(),
        name: item.name.clone(),
        contact_email_text: item.contact_email.clone(),
        sequence: item.sequence.to_string(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_policy_estimate_views(items: &[DefaultPolicyEstimate]) -> Vec<DefaultPolicyEstimateView> {
    let mut views: Vec<DefaultPolicyEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_policy_estimate_view(item));
    }
    views
}

pub fn sum_default_policy_estimate_view(view: &DefaultPolicyEstimateView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_policy_estimate("code-318", "title-600");
        let view = to_default_policy_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_policy_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
