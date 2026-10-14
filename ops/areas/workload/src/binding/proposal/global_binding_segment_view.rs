use super::*;

use acme_kit::text;

/// How screens and exports show a global binding segment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalBindingSegmentView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub burst_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_global_binding_segment_view(item: &GlobalBindingSegment) -> GlobalBindingSegmentView {
    GlobalBindingSegmentView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        burst_text: item.burst.to_string(),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_binding_segment_views(items: &[GlobalBindingSegment]) -> Vec<GlobalBindingSegmentView> {
    let mut views: Vec<GlobalBindingSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_binding_segment_view(item));
    }
    views
}

pub fn sum_global_binding_segment_view(view: &GlobalBindingSegmentView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_rate + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_binding_segment("code-634", "title-758");
        let view = to_global_binding_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_binding_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
