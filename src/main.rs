use std::error::Error;
use std::time::Duration;
use aws_config::meta::region::RegionProviderChain;
use aws_config::meta::credentials::CredentialsProviderChain;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_config::profile::ProfileFileRegionProvider;
use aws_config::provider_config::ProviderConfig;
use aws_types::Credentials;
use aws_config::default_provider::credentials;
use aws_config::default_provider::region;
use aws_config::profile::mfa_token::{MfaToken, MfaTokenFetchError, ProvideMfaToken, future::ProvideMfaToken as ProvideMfaFuture};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct ProvideMfaTokenFromStdin();

#[derive(Error, Debug)]
enum ProvideMfaTokenFromStdinError {
    #[error("stdin is not a tty; cannot read MFA token input")]
    NotATty,
}

impl ProvideMfaToken for ProvideMfaTokenFromStdin {
    fn mfa_token(&self, serial: &str) -> ProvideMfaFuture {
        if !atty::is(atty::Stream::Stdin) {
            return ProvideMfaFuture::ready(
                Err(MfaTokenFetchError::provider_error(ProvideMfaTokenFromStdinError::NotATty)));
        }
        match rpassword::prompt_password(format!("Enter MFA code for {}: ", serial)) {
            Err(e) => {
                ProvideMfaFuture::ready(Err(MfaTokenFetchError::provider_error(e)))
            }
            Ok(t) => {
                ProvideMfaFuture::ready(Ok(MfaToken::from(t)))
            }
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let creds =
        credentials::DefaultCredentialsChain::builder()
            .profile_name("home")
            .mfa_token(ProvideMfaTokenFromStdin())
            .load_timeout(Duration::from_secs(120))
            .build();
    let region = region::Builder::default().profile_name("home").build();
    let config = aws_config::from_env()
        .credentials_provider(creds.await)
        .region(region)
        .load()
        .await;
    let sts_client = aws_sdk_sts::Client::new(&config);
    let session_token = sts_client.get_session_token().send().await.unwrap();
    let credentials = session_token.credentials.unwrap();
    println!("AWS_SESSION_TOKEN={}",credentials.session_token.unwrap());
    Ok(())
}
