use super::*;

/// How screens and exports show a lenient release policy.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReleasePolicyView {
    pub code_text: String,
    pub label: String,
    pub burst: String,
    pub approved: String,
    pub floor_value: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_lenient_release_policy_view(item: &LenientReleasePolicy) -> LenientReleasePolicyView {
    LenientReleasePolicyView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        burst: item.burst.to_string(),
        approved: format!("{}", item.approved),
        floor_value: format!("{:.2}", item.floor_value),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_release_policy_views(items: &[LenientReleasePolicy]) -> Vec<LenientReleasePolicyView> {
    let mut views: Vec<LenientReleasePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_release_policy_view(item));
    }
    views
}

pub fn sum_lenient_release_policy_view(view: &LenientReleasePolicyView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_burst as f64 + parsed_floor_value + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_release_policy("code-227", "label-292");
        let view = to_lenient_release_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_release_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
