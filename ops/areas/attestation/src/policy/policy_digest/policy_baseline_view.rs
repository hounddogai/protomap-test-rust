use super::*;

use acme_kit::text;

/// How screens and exports show a policy baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBaselineView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_policy_baseline_view(item: &PolicyBaseline) -> PolicyBaselineView {
    PolicyBaselineView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        fingerprint: text::truncate(&item.fingerprint, 80),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_baseline_views(items: &[PolicyBaseline]) -> Vec<PolicyBaselineView> {
    let mut views: Vec<PolicyBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_baseline_view(item));
    }
    views
}

pub fn sum_policy_baseline_view(view: &PolicyBaselineView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_baseline("code-868", "title-161");
        let view = to_policy_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
