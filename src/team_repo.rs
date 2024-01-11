//! The code in this module interacts with the
//! <https://github.com/rust-lang/team/> repository.
//!
//! The [TeamRepo] trait is used to abstract away the HTTP client for testing
//! purposes. The [TeamRepoImpl] struct is the actual implementation of
//! the trait.

use crate::certs;
use async_trait::async_trait;
use mockall::automock;
use reqwest::{Certificate, Client};

#[automock]
#[async_trait]
pub trait TeamRepo {
    async fn get_team(&self, name: &str) -> anyhow::Result<Team>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    pub name: String,
    pub kind: String,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Member {
    pub name: String,
    pub github: String,
    pub github_id: i32,
    pub is_lead: bool,
}

pub struct TeamRepoImpl {
    client: Client,
}

impl TeamRepoImpl {
    fn new(client: Client) -> Self {
        TeamRepoImpl { client }
    }
}

impl Default for TeamRepoImpl {
    fn default() -> Self {
        let client = build_client();
        TeamRepoImpl::new(client)
    }
}

fn build_client() -> Client {
    let x1_cert = Certificate::from_pem(certs::ISRG_ROOT_X1).unwrap();
    let x2_cert = Certificate::from_pem(certs::ISRG_ROOT_X2).unwrap();

    Client::builder()
        .tls_built_in_root_certs(false)
        .add_root_certificate(x1_cert)
        .add_root_certificate(x2_cert)
        .build()
        .unwrap()
}

#[async_trait]
impl TeamRepo for TeamRepoImpl {
    async fn get_team(&self, name: &str) -> anyhow::Result<Team> {
        let url = format!("https://team-api.infra.rust-lang.org/v1/teams/{name}.json");
        let response = self.client.get(url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::team_repo::build_client;

    /// This test is here to make sure that the client is built
    /// correctly without panicking.
    #[test]
    fn test_build_client() {
        let _client = build_client();
    }
}
