use super::*;

use acme_kit::text;

/// How screens and exports show a region pin group.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroupView {
    pub code: String,
    pub title_text: String,
    pub cipher_text: String,
    pub timezone: String,
    pub revision: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_region_pin_group_view(item: &RegionPinGroup) -> RegionPinGroupView {
    RegionPinGroupView {
        code: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 64),
        cipher_text: item.cipher.clone(),
        timezone: item.timezone.clone(),
        revision: item.revision.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_group_views(items: &[RegionPinGroup]) -> Vec<RegionPinGroupView> {
    let mut views: Vec<RegionPinGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_group_view(item));
    }
    views
}

pub fn sum_region_pin_group_view(view: &RegionPinGroupView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_revision as f64 + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_group("code-288", "title-121");
        let view = to_region_pin_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
