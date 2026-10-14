use super::*;

/// How screens and exports show a page override.
#[derive(Debug, Clone, PartialEq)]
pub struct PageOverrideView {
    pub code: String,
    pub label_text: String,
    pub batch_size: String,
    pub key_id: String,
    pub ratio: String,
    pub region: String,
    pub quantity_text: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_page_override_view(item: &PageOverride) -> PageOverrideView {
    PageOverrideView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        key_id: item.key_id.clone(),
        ratio: format!("{:.2}", item.ratio),
        region: item.region.clone(),
        quantity_text: item.quantity.to_string(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_override_views(items: &[PageOverride]) -> Vec<PageOverrideView> {
    let mut views: Vec<PageOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_override_view(item));
    }
    views
}

pub fn sum_page_override_view(view: &PageOverrideView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_ratio + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_override("code-149", "label-528");
        let view = to_page_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
