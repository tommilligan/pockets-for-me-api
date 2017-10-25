pub type ElasticId = String;

pub mod elastic;
pub mod query;
pub mod response;


use elastic::client::responses::parse::{HttpResponseHead, IsOk, MaybeOkResponse, ResponseBody, Unbuffered};
use elastic::client::responses::parse::ParseResponseError;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Shards {
    total: u32,
    successful: u32,
    failed: u32,
}

#[derive(Deserialize, Debug)]
pub struct SuggestResponse {
    #[serde(rename = "_shards")] shards: Shards,
    qux: Vec<Sugestee>
}

impl SuggestResponse {
    pub fn inner(&self) -> &Vec<Sugestee> {
        &self.qux
    }
}

impl IsOk for SuggestResponse {
    fn is_ok<B: ResponseBody>(head: HttpResponseHead, body: Unbuffered<B>) -> Result<MaybeOkResponse<B>, ParseResponseError> {
        match head.status() {
            200...299 => Ok(MaybeOkResponse::ok(body)),
            _ => Ok(MaybeOkResponse::err(body)),
        }
    }
}

/** Full metadata and source for a single hit. */
#[derive(Deserialize, Debug)]
pub struct Sugestee {
    text: String,
    offset: u64,
    length: u64,
    #[serde(rename = "options")] suggestions: Vec<Suggestion>
}

impl Sugestee {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn suggestions(&self) -> &Vec<Suggestion> {
        &self.suggestions
    }
}

#[derive(Deserialize, Debug)]
pub struct Suggestion {
    text: String,
    score: f32,
    freq: u64
}

impl Suggestion {
    /** The text of the suggestion. */
    pub fn text(&self) -> &str {
        &self.text
    }

    /** The score of the suggestion. */
    pub fn score(&self) -> f32 {
        self.score.clone()
    }

    /** The frequency of the suggestion. */
    pub fn freq(&self) -> u64 {
        self.freq.clone()
    }
}

