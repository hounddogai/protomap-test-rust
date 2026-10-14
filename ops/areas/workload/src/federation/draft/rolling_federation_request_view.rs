use super::*;

/// How screens and exports show a rolling federation request.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationRequestView {
    pub code_text: String,
    pub title_text: String,
    pub maximum: String,
    pub currency_text: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_rolling_federation_request_view(item: &RollingFederationRequest) -> RollingFederationRequestView {
    RollingFederationRequestView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        maximum: format!("{:.2}", item.maximum),
        currency_text: item.currency.clone(),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_federation_request_views(items: &[RollingFederationRequest]) -> Vec<RollingFederationRequestView> {
    let mut views: Vec<RollingFederationRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_federation_request_view(item));
    }
    views
}

pub fn sum_rolling_federation_request_view(view: &RollingFederationRequestView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_federation_request("code-446", "title-598");
        let view = to_rolling_federation_request_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_federation_request_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
