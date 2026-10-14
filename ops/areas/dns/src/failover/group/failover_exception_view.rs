use super::*;

use acme_kit::text;

/// How screens and exports show a failover exception.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverExceptionView {
    pub code: String,
    pub title_text: String,
    pub scope: String,
    pub margin: String,
    pub audience: String,
    pub namespace_text: String,
    pub external_ref_text: String,
    pub cost: String,
    pub retries: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_failover_exception_view(item: &FailoverException) -> FailoverExceptionView {
    FailoverExceptionView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        scope: item.scope.clone(),
        margin: format!("{:.2}", item.margin),
        audience: item.audience.clone(),
        namespace_text: item.namespace.clone(),
        external_ref_text: text::truncate(&item.external_ref, 40),
        cost: format!("{:.2}", item.cost),
        retries: item.retries.to_string(),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_exception_views(items: &[FailoverException]) -> Vec<FailoverExceptionView> {
    let mut views: Vec<FailoverExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_exception_view(item));
    }
    views
}

pub fn sum_failover_exception_view(view: &FailoverExceptionView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_margin + parsed_cost + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_exception("code-146", "title-277");
        let view = to_failover_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
