use super::*;

/// How screens and exports show a pending query bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryBundleView {
    pub code_text: String,
    pub title: String,
    pub cipher_text: String,
    pub discount: String,
    pub retries: String,
    pub audience_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_pending_query_bundle_view(item: &PendingQueryBundle) -> PendingQueryBundleView {
    PendingQueryBundleView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cipher_text: item.cipher.clone(),
        discount: format!("{:.2}", item.discount),
        retries: item.retries.to_string(),
        audience_text: item.audience.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_query_bundle_views(items: &[PendingQueryBundle]) -> Vec<PendingQueryBundleView> {
    let mut views: Vec<PendingQueryBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_query_bundle_view(item));
    }
    views
}

pub fn sum_pending_query_bundle_view(view: &PendingQueryBundleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_discount + (parsed_retries as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_query_bundle("code-563", "title-987");
        let view = to_pending_query_bundle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_query_bundle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
