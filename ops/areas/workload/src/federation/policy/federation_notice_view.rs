use super::*;

use acme_kit::text;

/// How screens and exports show a federation notice.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeView {
    pub code: String,
    pub title_text: String,
    pub weight_text: String,
    pub limit: String,
    pub duration_minutes_text: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_federation_notice_view(item: &FederationNotice) -> FederationNoticeView {
    FederationNoticeView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        weight_text: format!("{:.2}", item.weight),
        limit: item.limit.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_notice_views(items: &[FederationNotice]) -> Vec<FederationNoticeView> {
    let mut views: Vec<FederationNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_notice_view(item));
    }
    views
}

pub fn sum_federation_notice_view(view: &FederationNoticeView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_weight + (parsed_limit as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_notice("code-8", "title-258");
        let view = to_federation_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
