use super::*;

/// How screens and exports show a freeze bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBundleView {
    pub code: String,
    pub title: String,
    pub maximum_text: String,
    pub floor_value: String,
    pub revision: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_freeze_bundle_view(item: &FreezeBundle) -> FreezeBundleView {
    FreezeBundleView {
        code: item.code.clone(),
        title: item.title.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        floor_value: format!("{:.2}", item.floor_value),
        revision: item.revision.to_string(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_bundle_views(items: &[FreezeBundle]) -> Vec<FreezeBundleView> {
    let mut views: Vec<FreezeBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_bundle_view(item));
    }
    views
}

pub fn sum_freeze_bundle_view(view: &FreezeBundleView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_maximum + parsed_floor_value + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_bundle("code-54", "title-324");
        let view = to_freeze_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
