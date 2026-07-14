use std::collections::HashMap;

use reqwest::Client;
use serde::Deserialize;

#[derive(Clone, Copy)]
pub struct PresetAsset {
    pub symbol: &'static str,
    pub name: &'static str,
}

pub const PRESET_ASSETS: [PresetAsset; 6] = [
    PresetAsset {
        symbol: "ABEV3",
        name: "Ambev",
    },
    PresetAsset {
        symbol: "PETR4",
        name: "Petrobras",
    },
    PresetAsset {
        symbol: "VALE3",
        name: "Vale",
    },
    PresetAsset {
        symbol: "ITUB4",
        name: "Itaú Unibanco",
    },
    PresetAsset {
        symbol: "BBDC4",
        name: "Bradesco",
    },
    PresetAsset {
        symbol: "BBAS3",
        name: "Banco do Brasil",
    },
];

#[derive(Debug, Deserialize)]
struct QuoteResponse {
    results: Vec<BrapiQuote>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BrapiQuote {
    pub symbol: String,
    pub data: Option<BrapiQuoteData>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BrapiQuoteData {
    #[serde(rename = "regularMarketPrice")]
    pub regular_market_price: Option<f64>,
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    #[serde(rename = "longName")]
    pub long_name: Option<String>,
}

pub async fn fetch_quotes(
    http_client: &Client,
    token: &str,
    symbols: &[String],
) -> Result<HashMap<String, BrapiQuote>, reqwest::Error> {
    if symbols.is_empty() {
        return Ok(HashMap::new());
    }

    let response = http_client
        .get("https://brapi.dev/api/v2/stocks/quote")
        .bearer_auth(token)
        .query(&[("symbols", symbols.join(","))])
        .send()
        .await?
        .error_for_status()?
        .json::<QuoteResponse>()
        .await?;

    Ok(response
        .results
        .into_iter()
        .map(|quote| (quote.symbol.clone(), quote))
        .collect())
}
