use super::*;

/// How screens and exports show a federation exception.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationExceptionView {
    pub code: String,
    pub title_text: String,
    pub burst_text: String,
    pub minimum: String,
    pub grace_days_text: String,
    pub port: String,
    pub ttl_seconds_text: String,
    pub quantity: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_federation_exception_view(item: &FederationException) -> FederationExceptionView {
    FederationExceptionView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        burst_text: item.burst.to_string(),
        minimum: format!("{:.2}", item.minimum),
        grace_days_text: item.grace_days.to_string(),
        port: item.port.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        quantity: item.quantity.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_exception_views(items: &[FederationException]) -> Vec<FederationExceptionView> {
    let mut views: Vec<FederationExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_exception_view(item));
    }
    views
}

pub fn sum_federation_exception_view(view: &FederationExceptionView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_burst as f64 + parsed_minimum + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_exception("code-905", "title-319");
        let view = to_federation_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
