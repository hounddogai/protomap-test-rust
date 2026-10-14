use super::*;

use acme_kit::text;

/// How screens and exports show a freeze mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMappingView {
    pub code_text: String,
    pub title: String,
    pub host_text: String,
    pub ratio: String,
    pub margin_text: String,
    pub grace_days_text: String,
    pub phone: String,
    pub checksum: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_freeze_mapping_view(item: &FreezeMapping) -> FreezeMappingView {
    FreezeMappingView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        host_text: item.host.clone(),
        ratio: format!("{:.2}", item.ratio),
        margin_text: format!("{:.2}", item.margin),
        grace_days_text: item.grace_days.to_string(),
        phone: text::truncate(&item.phone, 80),
        checksum: item.checksum.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_mapping_views(items: &[FreezeMapping]) -> Vec<FreezeMappingView> {
    let mut views: Vec<FreezeMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_mapping_view(item));
    }
    views
}

pub fn sum_freeze_mapping_view(view: &FreezeMappingView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ratio + parsed_margin + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_mapping("code-68", "title-847");
        let view = to_freeze_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
