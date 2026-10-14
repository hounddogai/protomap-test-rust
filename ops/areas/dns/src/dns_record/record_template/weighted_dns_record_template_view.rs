use super::*;

use acme_kit::text;

/// How screens and exports show a weighted dns record template.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDnsRecordTemplateView {
    pub code: String,
    pub title_text: String,
    pub phone: String,
    pub locale: String,
    pub batch_size: String,
    pub score_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_weighted_dns_record_template_view(item: &WeightedDnsRecordTemplate) -> WeightedDnsRecordTemplateView {
    WeightedDnsRecordTemplateView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        phone: text::truncate(&item.phone, 64),
        locale: item.locale.clone(),
        batch_size: item.batch_size.to_string(),
        score_text: format!("{:.2}", item.score),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_dns_record_template_views(
    items: &[WeightedDnsRecordTemplate],
) -> Vec<WeightedDnsRecordTemplateView> {
    let mut views: Vec<WeightedDnsRecordTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_dns_record_template_view(item));
    }
    views
}

pub fn sum_weighted_dns_record_template_view(view: &WeightedDnsRecordTemplateView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_dns_record_template("code-597", "title-452");
        let view = to_weighted_dns_record_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_dns_record_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
