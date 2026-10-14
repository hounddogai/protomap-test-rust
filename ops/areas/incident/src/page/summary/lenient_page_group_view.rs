use super::*;

use acme_kit::text;

/// How screens and exports show a lenient page group.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroupView {
    pub code: String,
    pub name: String,
    pub priority: String,
    pub archived_text: String,
    pub contact_email_text: String,
    pub batch_size_text: String,
    pub issuer_text: String,
    pub port: String,
    pub cost_text: String,
    pub owner_id_text: String,
    pub status_label: String,
}

pub fn to_lenient_page_group_view(item: &LenientPageGroup) -> LenientPageGroupView {
    LenientPageGroupView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        priority: item.priority.to_string(),
        archived_text: format!("{}", item.archived),
        contact_email_text: item.contact_email.clone(),
        batch_size_text: item.batch_size.to_string(),
        issuer_text: item.issuer.clone(),
        port: item.port.to_string(),
        cost_text: format!("{:.2}", item.cost),
        owner_id_text: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_page_group_views(items: &[LenientPageGroup]) -> Vec<LenientPageGroupView> {
    let mut views: Vec<LenientPageGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_page_group_view(item));
    }
    views
}

pub fn sum_lenient_page_group_view(view: &LenientPageGroupView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_batch_size as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_page_group("code-593", "name-902");
        let view = to_lenient_page_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_page_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
