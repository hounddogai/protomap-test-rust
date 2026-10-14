use super::*;

/// How screens and exports show a freeze manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeManifestView {
    pub code: String,
    pub title: String,
    pub maximum_text: String,
    pub starts_at_text: String,
    pub amount: String,
    pub cost: String,
    pub visible_text: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_freeze_manifest_view(item: &FreezeManifest) -> FreezeManifestView {
    FreezeManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        starts_at_text: item.starts_at.to_string(),
        amount: format!("{:.2}", item.amount),
        cost: format!("{:.2}", item.cost),
        visible_text: format!("{}", item.visible),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_manifest_views(items: &[FreezeManifest]) -> Vec<FreezeManifestView> {
    let mut views: Vec<FreezeManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_manifest_view(item));
    }
    views
}

pub fn sum_freeze_manifest_view(view: &FreezeManifestView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_maximum + (parsed_starts_at as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_manifest("code-844", "title-276");
        let view = to_freeze_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
