use super::*;

use acme_kit::text;

/// How screens and exports show a grant record.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRecordView {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub description_text: String,
    pub burst: String,
    pub rate_text: String,
    pub approved: String,
    pub duration_minutes: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_grant_record_view(item: &GrantRecord) -> GrantRecordView {
    GrantRecordView {
        code: item.code.clone(),
        label: item.label.clone(),
        issuer: text::truncate(&item.issuer, 40),
        description_text: text::truncate(&item.description, 80),
        burst: item.burst.to_string(),
        rate_text: format!("{:.2}", item.rate),
        approved: format!("{}", item.approved),
        duration_minutes: item.duration_minutes.to_string(),
        owner_id_text: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_record_views(items: &[GrantRecord]) -> Vec<GrantRecordView> {
    let mut views: Vec<GrantRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_record_view(item));
    }
    views
}

pub fn sum_grant_record_view(view: &GrantRecordView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_burst as f64 + parsed_rate + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_record("code-800", "label-765");
        let view = to_grant_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
