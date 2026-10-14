use super::*;

/// How screens and exports show a isolation target.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTargetView {
    pub code_text: String,
    pub title_text: String,
    pub source: String,
    pub region: String,
    pub segment: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_isolation_target_view(item: &IsolationTarget) -> IsolationTargetView {
    IsolationTargetView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        source: item.source.clone(),
        region: item.region.clone(),
        segment: item.segment.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_target_views(items: &[IsolationTarget]) -> Vec<IsolationTargetView> {
    let mut views: Vec<IsolationTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_target_view(item));
    }
    views
}

pub fn sum_isolation_target_view(view: &IsolationTargetView) -> Result<f64, String> {
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
        let item = new_isolation_target("code-636", "title-341");
        let view = to_isolation_target_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_target_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
