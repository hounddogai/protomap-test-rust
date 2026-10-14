use super::*;

use acme_kit::text;

/// How screens and exports show a slo group.
#[derive(Debug, Clone, PartialEq)]
pub struct SloGroupView {
    pub code_text: String,
    pub label_text: String,
    pub checksum: String,
    pub discount: String,
    pub batch_size: String,
    pub namespace_text: String,
    pub cipher: String,
    pub port_text: String,
    pub quantity_text: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_slo_group_view(item: &SloGroup) -> SloGroupView {
    SloGroupView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        checksum: text::truncate(&item.checksum, 40),
        discount: format!("{:.2}", item.discount),
        batch_size: item.batch_size.to_string(),
        namespace_text: text::truncate(&item.namespace, 120),
        cipher: text::truncate(&item.cipher, 40),
        port_text: item.port.to_string(),
        quantity_text: item.quantity.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_group_views(items: &[SloGroup]) -> Vec<SloGroupView> {
    let mut views: Vec<SloGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_group_view(item));
    }
    views
}

pub fn sum_slo_group_view(view: &SloGroupView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_discount + (parsed_batch_size as f64) + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_slo_group("code-948", "label-918");
        let view = to_slo_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_slo_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
