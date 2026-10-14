use super::*;

use acme_kit::text;

/// How screens and exports show a primary query profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQueryProfileView {
    pub code: String,
    pub title: String,
    pub port_text: String,
    pub retries_text: String,
    pub visible: String,
    pub amount: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_primary_query_profile_view(item: &PrimaryQueryProfile) -> PrimaryQueryProfileView {
    PrimaryQueryProfileView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        port_text: item.port.to_string(),
        retries_text: item.retries.to_string(),
        visible: format!("{}", item.visible),
        amount: format!("{:.2}", item.amount),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_query_profile_views(items: &[PrimaryQueryProfile]) -> Vec<PrimaryQueryProfileView> {
    let mut views: Vec<PrimaryQueryProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_query_profile_view(item));
    }
    views
}

pub fn sum_primary_query_profile_view(view: &PrimaryQueryProfileView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_port as f64 + (parsed_retries as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_query_profile("code-337", "title-318");
        let view = to_primary_query_profile_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_query_profile_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
