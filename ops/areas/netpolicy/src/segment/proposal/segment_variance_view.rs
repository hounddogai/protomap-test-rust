use super::*;

use acme_kit::text;

/// How screens and exports show a segment variance.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVarianceView {
    pub code_text: String,
    pub title: String,
    pub burst_text: String,
    pub region: String,
    pub rate: String,
    pub margin: String,
    pub host: String,
    pub locale_text: String,
    pub port: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_segment_variance_view(item: &SegmentVariance) -> SegmentVarianceView {
    SegmentVarianceView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        burst_text: item.burst.to_string(),
        region: text::truncate(&item.region, 64),
        rate: format!("{:.2}", item.rate),
        margin: format!("{:.2}", item.margin),
        host: item.host.clone(),
        locale_text: item.locale.clone(),
        port: item.port.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_variance_views(items: &[SegmentVariance]) -> Vec<SegmentVarianceView> {
    let mut views: Vec<SegmentVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_variance_view(item));
    }
    views
}

pub fn sum_segment_variance_view(view: &SegmentVarianceView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_burst as f64 + parsed_rate + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_variance("code-683", "title-627");
        let view = to_segment_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
