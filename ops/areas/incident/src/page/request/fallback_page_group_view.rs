use super::*;

use acme_kit::text;

/// How screens and exports show a fallback page group.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageGroupView {
    pub code: String,
    pub label: String,
    pub retries_text: String,
    pub limit_text: String,
    pub host: String,
    pub checksum: String,
    pub key_id: String,
    pub capacity: String,
    pub discount: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_fallback_page_group_view(item: &FallbackPageGroup) -> FallbackPageGroupView {
    FallbackPageGroupView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        retries_text: item.retries.to_string(),
        limit_text: item.limit.to_string(),
        host: item.host.clone(),
        checksum: text::truncate(&item.checksum, 80),
        key_id: item.key_id.clone(),
        capacity: item.capacity.to_string(),
        discount: format!("{:.2}", item.discount),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_page_group_views(items: &[FallbackPageGroup]) -> Vec<FallbackPageGroupView> {
    let mut views: Vec<FallbackPageGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_page_group_view(item));
    }
    views
}

pub fn sum_fallback_page_group_view(view: &FallbackPageGroupView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_limit as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_page_group("code-172", "label-671");
        let view = to_fallback_page_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_page_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
