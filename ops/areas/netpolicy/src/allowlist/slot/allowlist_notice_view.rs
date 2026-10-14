use super::*;

/// How screens and exports show a allowlist notice.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistNoticeView {
    pub code: String,
    pub name: String,
    pub visible_text: String,
    pub created_at_text: String,
    pub maximum: String,
    pub checksum: String,
    pub segment: String,
    pub weight: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_allowlist_notice_view(item: &AllowlistNotice) -> AllowlistNoticeView {
    AllowlistNoticeView {
        code: item.code.clone(),
        name: item.name.clone(),
        visible_text: format!("{}", item.visible),
        created_at_text: item.created_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        checksum: item.checksum.clone(),
        segment: item.segment.clone(),
        weight: format!("{:.2}", item.weight),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_notice_views(items: &[AllowlistNotice]) -> Vec<AllowlistNoticeView> {
    let mut views: Vec<AllowlistNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_notice_view(item));
    }
    views
}

pub fn sum_allowlist_notice_view(view: &AllowlistNoticeView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_maximum + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_notice("code-435", "name-152");
        let view = to_allowlist_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
