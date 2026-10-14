use super::*;

use acme_kit::text;

/// How screens and exports show a default rollback record.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecordView {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub description_text: String,
    pub discount_text: String,
    pub port_text: String,
    pub batch_size: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_default_rollback_record_view(item: &DefaultRollbackRecord) -> DefaultRollbackRecordView {
    DefaultRollbackRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        cipher: item.cipher.clone(),
        description_text: text::truncate(&item.description, 80),
        discount_text: format!("{:.2}", item.discount),
        port_text: item.port.to_string(),
        batch_size: item.batch_size.to_string(),
        audience: text::truncate(&item.audience, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_rollback_record_views(items: &[DefaultRollbackRecord]) -> Vec<DefaultRollbackRecordView> {
    let mut views: Vec<DefaultRollbackRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_rollback_record_view(item));
    }
    views
}

pub fn sum_default_rollback_record_view(view: &DefaultRollbackRecordView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_discount + (parsed_port as f64) + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_rollback_record("code-655", "name-955");
        let view = to_default_rollback_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_rollback_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
