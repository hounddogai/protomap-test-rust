use super::*;

use acme_kit::text;

/// How screens and exports show a page quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PageQuotaView {
    pub code: String,
    pub title_text: String,
    pub ratio: String,
    pub summary: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_page_quota_view(item: &PageQuota) -> PageQuotaView {
    PageQuotaView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        summary: item.summary.clone(),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_quota_views(items: &[PageQuota]) -> Vec<PageQuotaView> {
    let mut views: Vec<PageQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_quota_view(item));
    }
    views
}

pub fn sum_page_quota_view(view: &PageQuotaView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_quota("code-893", "title-453");
        let view = to_page_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
