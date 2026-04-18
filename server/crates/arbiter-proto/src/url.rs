use base64::{Engine as _, prelude::BASE64_URL_SAFE};
use rustls_pki_types::CertificateDer;
use std::fmt::Display;

const ARBITER_URL_SCHEME: &str = "arbiter";
const CERT_QUERY_KEY: &str = "cert";
const BOOTSTRAP_TOKEN_QUERY_KEY: &str = "bootstrap_token";

#[derive(Debug, Clone)]
pub struct ArbiterUrl {
    pub host: String,
    pub port: u16,
    pub ca_cert: CertificateDer<'static>,
    pub bootstrap_token: Option<String>,
}

impl Display for ArbiterUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut base = format!(
            "{ARBITER_URL_SCHEME}://{}:{}?{CERT_QUERY_KEY}={}",
            self.host,
            self.port,
            BASE64_URL_SAFE.encode(&self.ca_cert)
        );
        if let Some(token) = &self.bootstrap_token {
            base.push_str(&format!("&{BOOTSTRAP_TOKEN_QUERY_KEY}={}", token));
        }
        f.write_str(&base)
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Invalid URL scheme, expected '{ARBITER_URL_SCHEME}://'")]
    #[diagnostic(
        code(arbiter::url::invalid_scheme),
        help("The URL must start with '{ARBITER_URL_SCHEME}://'")
    )]
    InvalidScheme,
    #[error("Missing host in URL")]
    #[diagnostic(
        code(arbiter::url::missing_host),
        help("The URL must include a host, e.g., '{ARBITER_URL_SCHEME}://127.0.0.1:<port>'")
    )]
    MissingHost,
    #[error("Missing port in URL")]
    #[diagnostic(
        code(arbiter::url::missing_port),
        help("The URL must include a port, e.g., '{ARBITER_URL_SCHEME}://127.0.0.1:1234'")
    )]
    MissingPort,
    #[error("Missing 'cert' query parameter in URL")]
    #[diagnostic(
        code(arbiter::url::missing_cert),
        help("The URL must include a 'cert' query parameter")
    )]
    MissingCert,
    #[error("Invalid base64 in 'cert' query parameter: {0}")]
    #[diagnostic(code(arbiter::url::invalid_cert_base64))]
    InvalidCertBase64(#[from] base64::DecodeError),
}

impl<'a> TryFrom<&'a str> for ArbiterUrl {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let url = url::Url::parse(value).map_err(|_| Error::InvalidScheme)?;

        if url.scheme() != ARBITER_URL_SCHEME {
            return Err(Error::InvalidScheme);
        }

        let host = url.host_str().ok_or(Error::MissingHost)?.to_string();
        let port = url.port().ok_or(Error::MissingPort)?;
        let cert_str = url
            .query_pairs()
            .find(|(k, _)| k == CERT_QUERY_KEY)
            .ok_or(Error::MissingCert)?
            .1;

        let cert = BASE64_URL_SAFE.decode(cert_str.as_ref())?;
        let cert = CertificateDer::from_slice(&cert).into_owned();

        let bootstrap_token = url
            .query_pairs()
            .find(|(k, _)| k == BOOTSTRAP_TOKEN_QUERY_KEY)
            .map(|(_, v)| v.to_string());

        Ok(ArbiterUrl {
            host,
            port,
            ca_cert: cert,
            bootstrap_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use rcgen::generate_simple_self_signed;
    use rstest::rstest;

    use super::*;

    #[rstest]

    fn test_parsing_correctness(
        #[values("127.0.0.1", "localhost", "192.168.1.1", "some.domain.com")] host: &str,

        #[values(None, Some("token123".to_string()))] bootstrap_token: Option<String>,
    ) {
        let cert = generate_simple_self_signed(&["Arbiter CA".into()]).unwrap();
        let cert = cert.cert.der();

        let url = ArbiterUrl {
            host: host.to_string(),
            port: 1234,
            ca_cert: cert.clone().into_owned(),
            bootstrap_token,
        };
        let url_str = url.to_string();
        let parsed_url = ArbiterUrl::try_from(url_str.as_str()).unwrap();
        assert_eq!(url.host, parsed_url.host);
        assert_eq!(url.port, parsed_url.port);
        assert_eq!(url.ca_cert.to_vec(), parsed_url.ca_cert.to_vec());
        assert_eq!(url.bootstrap_token, parsed_url.bootstrap_token);
    }
}
