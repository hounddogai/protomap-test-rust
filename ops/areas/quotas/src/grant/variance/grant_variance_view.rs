use super::*;

/// How screens and exports show a grant variance.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantVarianceView {
    pub code_text: String,
    pub title: String,
    pub ratio: String,
    pub capacity: String,
    pub amount: String,
    pub attempts_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_grant_variance_view(item: &GrantVariance) -> GrantVarianceView {
    GrantVarianceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        capacity: item.capacity.to_string(),
        amount: format!("{:.2}", item.amount),
        attempts_text: item.attempts.to_string(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_variance_views(items: &[GrantVariance]) -> Vec<GrantVarianceView> {
    let mut views: Vec<GrantVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_variance_view(item));
    }
    views
}

pub fn sum_grant_variance_view(view: &GrantVarianceView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ratio + (parsed_capacity as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_variance("code-71", "title-621");
        let view = to_grant_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
