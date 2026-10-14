use super::*;

use acme_kit::text;

/// How screens and exports show a migration notice.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationNoticeView {
    pub code_text: String,
    pub title: String,
    pub pinned: String,
    pub quantity: String,
    pub source_text: String,
    pub ratio: String,
    pub maximum: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_migration_notice_view(item: &MigrationNotice) -> MigrationNoticeView {
    MigrationNoticeView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        pinned: format!("{}", item.pinned),
        quantity: item.quantity.to_string(),
        source_text: item.source.clone(),
        ratio: format!("{:.2}", item.ratio),
        maximum: format!("{:.2}", item.maximum),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_notice_views(items: &[MigrationNotice]) -> Vec<MigrationNoticeView> {
    let mut views: Vec<MigrationNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_notice_view(item));
    }
    views
}

pub fn sum_migration_notice_view(view: &MigrationNoticeView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_ratio + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_notice("code-9", "title-199");
        let view = to_migration_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
