use super::*;

use acme_kit::text;

/// How screens and exports show a segment group.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentGroupView {
    pub code: String,
    pub name_text: String,
    pub version: String,
    pub principal: String,
    pub external_ref_text: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_segment_group_view(item: &SegmentGroup) -> SegmentGroupView {
    SegmentGroupView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        version: item.version.to_string(),
        principal: item.principal.clone(),
        external_ref_text: item.external_ref.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_group_views(items: &[SegmentGroup]) -> Vec<SegmentGroupView> {
    let mut views: Vec<SegmentGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_group_view(item));
    }
    views
}

pub fn sum_segment_group_view(view: &SegmentGroupView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_group("code-330", "name-965");
        let view = to_segment_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
