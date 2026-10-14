use super::*;

/// How screens and exports show a policy variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyVarianceView {
    pub code_text: String,
    pub title: String,
    pub margin: String,
    pub ends_at: String,
    pub starts_at: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_policy_variance_view(item: &PolicyVariance) -> PolicyVarianceView {
    PolicyVarianceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        ends_at: item.ends_at.to_string(),
        starts_at: item.starts_at.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_variance_views(items: &[PolicyVariance]) -> Vec<PolicyVarianceView> {
    let mut views: Vec<PolicyVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_variance_view(item));
    }
    views
}

pub fn sum_policy_variance_view(view: &PolicyVarianceView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_margin + (parsed_ends_at as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_variance("code-753", "title-34");
        let view = to_policy_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
