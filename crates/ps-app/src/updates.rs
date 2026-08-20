//! Fetches the latest GitHub Release JSON for the optional update check.

use std::io::Read;
use std::time::Duration;

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/mpakus/1537paperstreet/releases/latest";
const USER_AGENT: &str = concat!(
    "1537paperstreet/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/mpakus/1537paperstreet)"
);
const MAX_BYTES: u64 = 256 * 1024;

/// Downloads the GitHub `releases/latest` JSON body.
pub(crate) fn fetch_latest_release_json() -> Result<String, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .redirects(0)
        .build();
    let response = agent
        .get(LATEST_RELEASE_URL)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call()
        .map_err(|_| unreachable_message())?;
    if response.status() != 200 {
        return Err(unreachable_message());
    }
    let mut body = String::new();
    response
        .into_reader()
        .take(MAX_BYTES + 1)
        .read_to_string(&mut body)
        .map_err(|_| unreachable_message())?;
    if body.len() as u64 > MAX_BYTES {
        return Err("GitHub did not return a release.".to_owned());
    }
    Ok(body)
}

fn unreachable_message() -> String {
    "Couldn't reach GitHub to check for updates.".to_owned()
}
