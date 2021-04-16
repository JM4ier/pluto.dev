use crate::config::CONFIG;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json;

#[derive(Clone, Deserialize, Debug)]
pub struct Member {
    pub title: String,
    pub url: String,
    pub feed: String,
}

const DATA_URL: &str = "https://xyquadrat.ch/polyring/data/members.json";

lazy_static! {
    pub static ref MEMBERS: Vec<Member> = {
        let response = ureq::get(DATA_URL)
            .call()
            .expect("fetching member data failed.")
            .into_string()
            .unwrap();
        serde_json::from_str(&response).expect("incorrect json format.")
    };
    pub static ref BANNER: String = format!(
        include_str!("polyring-banner.html"),
        prev = prev_next().0,
        next = prev_next().1,
        member_count = MEMBERS.len(),
    );
}

fn prev_next() -> (String, String) {
    let url = &CONFIG.url;
    for (i, member) in MEMBERS.iter().enumerate() {
        if &member.url == url {
            let prev = MEMBERS[(i + MEMBERS.len() - 1) % MEMBERS.len()].url.clone();
            let next = MEMBERS[(i + 1) % MEMBERS.len()].url.clone();
            return (prev, next);
        }
    }
    panic!("I'm apparently not in polyring anymore :(");
}
