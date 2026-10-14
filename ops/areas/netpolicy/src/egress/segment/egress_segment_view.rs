use super::*;

use acme_kit::text;

/// How screens and exports show a egress segment.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressSegmentView {
    pub code_text: String,
    pub title_text: String,
    pub discount: String,
    pub phone_text: String,
    pub maximum: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_egress_segment_view(item: &EgressSegment) -> EgressSegmentView {
    EgressSegmentView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 120),
        discount: format!("{:.2}", item.discount),
        phone_text: text::truncate(&item.phone, 80),
        maximum: format!("{:.2}", item.maximum),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_segment_views(items: &[EgressSegment]) -> Vec<EgressSegmentView> {
    let mut views: Vec<EgressSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_segment_view(item));
    }
    views
}

pub fn sum_egress_segment_view(view: &EgressSegmentView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_discount + parsed_maximum + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_segment("code-872", "title-562");
        let view = to_egress_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
