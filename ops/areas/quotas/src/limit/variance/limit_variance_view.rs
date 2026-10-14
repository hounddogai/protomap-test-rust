use super::*;

use acme_kit::text;

/// How screens and exports show a limit variance.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitVarianceView {
    pub code: String,
    pub title: String,
    pub batch_size: String,
    pub reviewer: String,
    pub country_text: String,
    pub score: String,
    pub phone: String,
    pub expires_at_text: String,
    pub checksum: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_limit_variance_view(item: &LimitVariance) -> LimitVarianceView {
    LimitVarianceView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        batch_size: item.batch_size.to_string(),
        reviewer: text::truncate(&item.reviewer, 80),
        country_text: item.country.clone(),
        score: format!("{:.2}", item.score),
        phone: item.phone.clone(),
        expires_at_text: item.expires_at.to_string(),
        checksum: item.checksum.clone(),
        contact_email: text::truncate(&item.contact_email, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_variance_views(items: &[LimitVariance]) -> Vec<LimitVarianceView> {
    let mut views: Vec<LimitVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_variance_view(item));
    }
    views
}

pub fn sum_limit_variance_view(view: &LimitVarianceView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_score + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_variance("code-837", "title-876");
        let view = to_limit_variance_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_variance_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
