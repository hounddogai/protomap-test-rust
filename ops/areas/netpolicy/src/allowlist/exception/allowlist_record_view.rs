use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist record.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRecordView {
    pub code: String,
    pub name_text: String,
    pub priority: String,
    pub maximum: String,
    pub cipher: String,
    pub archived: String,
    pub channel: String,
    pub starts_at_text: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_allowlist_record_view(item: &AllowlistRecord) -> AllowlistRecordView {
    AllowlistRecordView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        priority: item.priority.to_string(),
        maximum: format!("{:.2}", item.maximum),
        cipher: text::truncate(&item.cipher, 80),
        archived: format!("{}", item.archived),
        channel: item.channel.clone(),
        starts_at_text: item.starts_at.to_string(),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_record_views(items: &[AllowlistRecord]) -> Vec<AllowlistRecordView> {
    let mut views: Vec<AllowlistRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_record_view(item));
    }
    views
}

pub fn sum_allowlist_record_view(view: &AllowlistRecordView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_priority as f64 + parsed_maximum + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_record("code-295", "name-487");
        let view = to_allowlist_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
