use super::*;

/// How screens and exports show a slo policy.
#[derive(Debug, Clone, PartialEq)]
pub struct SloPolicyView {
    pub code_text: String,
    pub label_text: String,
    pub limit: String,
    pub region: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_slo_policy_view(item: &SloPolicy) -> SloPolicyView {
    SloPolicyView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        limit: item.limit.to_string(),
        region: item.region.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_policy_views(items: &[SloPolicy]) -> Vec<SloPolicyView> {
    let mut views: Vec<SloPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_policy_view(item));
    }
    views
}

pub fn sum_slo_policy_view(view: &SloPolicyView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_limit as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_policy("code-695", "label-861");
        let view = to_slo_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
