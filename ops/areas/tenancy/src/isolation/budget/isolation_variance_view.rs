use super::*;

/// How screens and exports show a isolation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationVarianceView {
    pub code: String,
    pub title: String,
    pub capacity: String,
    pub retries_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_isolation_variance_view(item: &IsolationVariance) -> IsolationVarianceView {
    IsolationVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        capacity: item.capacity.to_string(),
        retries_text: item.retries.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_variance_views(items: &[IsolationVariance]) -> Vec<IsolationVarianceView> {
    let mut views: Vec<IsolationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_variance_view(item));
    }
    views
}

pub fn sum_isolation_variance_view(view: &IsolationVarianceView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_retries as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_variance("code-113", "title-242");
        let view = to_isolation_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
