//! HTTP transport. The client talks to the gateway through the [`Transport`]
//! trait so tests (and callers with special needs) can replace the network.
//! The default implementation, [`UreqTransport`], uses `ureq` with rustls.

use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

/// One HTTP request as the client wants it sent.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpRequest {
    /// `GET`, `POST`, `PUT` or `DELETE`.
    pub method: String,
    /// Absolute URL including any query string.
    pub url: String,
    /// Request headers, in order.
    pub headers: Vec<(String, String)>,
    /// JSON body for POST/PUT, `None` for GET/DELETE.
    pub body: Option<String>,
    /// Total timeout (php-dna: 30 s).
    pub timeout: Duration,
    /// Connect timeout (php-dna: 10 s).
    pub connect_timeout: Duration,
    /// Verify the server's TLS certificate.
    pub verify_tls: bool,
}

/// The gateway's answer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Raw response body.
    pub body: String,
    /// Response headers (lower-cased names).
    pub headers: Vec<(String, String)>,
}

/// A failure below HTTP: timeout, DNS, connection refused, TLS, reset.
///
/// `code` follows curl's error numbers so error envelopes match php-dna
/// (`CURL_28` for a timeout, `CURL_6` for DNS, `CURL_7` for a refused
/// connection, `CURL_35` for TLS, `CURL_56` for a broken connection).
#[derive(Debug, Clone, PartialEq)]
pub struct TransportError {
    /// curl-compatible error number.
    pub code: i64,
    /// Human readable description.
    pub message: String,
}

impl TransportError {
    /// Convenience constructor.
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Whether this is a timeout (curl error 28).
    pub fn is_timeout(&self) -> bool {
        self.code == 28
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (curl {})", self.message, self.code)
    }
}

impl std::error::Error for TransportError {}

/// Sends HTTP requests. Implement it to plug in another HTTP client or a
/// fake for tests.
pub trait Transport: Send {
    /// Perform the request. Any HTTP status (including 4xx/5xx and 3xx) is a
    /// successful `Ok(HttpResponse)`; only failures below HTTP are errors.
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError>;
}

/// Default transport: blocking `ureq` with rustls. Redirects are not followed
/// (a 301/302 from the gateway means bad credentials, as in php-dna).
#[derive(Default)]
pub struct UreqTransport {
    verified: OnceLock<ureq::Agent>,
    unverified: OnceLock<ureq::Agent>,
}

impl UreqTransport {
    /// Creates the transport. Agents are built lazily on first use.
    pub fn new() -> Self {
        Self::default()
    }

    fn agent(&self, request: &HttpRequest) -> &ureq::Agent {
        let cell = if request.verify_tls {
            &self.verified
        } else {
            &self.unverified
        };
        cell.get_or_init(|| {
            build_agent(request.timeout, request.connect_timeout, request.verify_tls)
        })
    }
}

pub(crate) fn build_agent(timeout: Duration, connect: Duration, verify_tls: bool) -> ureq::Agent {
    let mut builder = ureq::AgentBuilder::new()
        .timeout(timeout)
        .timeout_connect(connect)
        .redirects(0);
    if !verify_tls {
        builder = builder.tls_config(Arc::new(insecure_tls_config()));
    }
    builder.build()
}

impl Transport for UreqTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let agent = self.agent(request);
        let mut call = agent.request(&request.method, &request.url);
        for (name, value) in &request.headers {
            call = call.set(name, value);
        }
        let result = match &request.body {
            Some(body) => call.send_string(body),
            None => call.call(),
        };
        let response = match result {
            Ok(response) => response,
            Err(ureq::Error::Status(_, response)) => response,
            Err(ureq::Error::Transport(err)) => return Err(map_transport_error(&err)),
        };
        let status = response.status();
        let headers = response
            .headers_names()
            .into_iter()
            .filter_map(|name| {
                response
                    .header(&name)
                    .map(|v| (name.to_ascii_lowercase(), v.to_string()))
            })
            .collect();
        let body = response
            .into_string()
            .map_err(|e| io_to_transport_error(&e))?;
        Ok(HttpResponse {
            status,
            body,
            headers,
        })
    }
}

fn io_to_transport_error(err: &std::io::Error) -> TransportError {
    use std::io::ErrorKind;
    match err.kind() {
        ErrorKind::TimedOut | ErrorKind::WouldBlock => {
            TransportError::new(28, format!("Operation timed out: {err}"))
        }
        _ => TransportError::new(
            56,
            format!("Failure when receiving data from the peer: {err}"),
        ),
    }
}

fn map_transport_error(err: &ureq::Transport) -> TransportError {
    use ureq::ErrorKind;
    let text = err.to_string();
    let lower = text.to_ascii_lowercase();
    if lower.contains("timed out") || lower.contains("timeout") {
        return TransportError::new(28, text);
    }
    let code = match err.kind() {
        ErrorKind::InvalidUrl => 3,
        ErrorKind::UnknownScheme => 1,
        ErrorKind::Dns => 6,
        ErrorKind::ConnectionFailed => {
            if lower.contains("certificate") || lower.contains("tls") || lower.contains("handshake")
            {
                35
            } else {
                7
            }
        }
        ErrorKind::TooManyRedirects => 47,
        ErrorKind::BadStatus | ErrorKind::BadHeader => 8,
        ErrorKind::ProxyConnect | ErrorKind::InvalidProxyUrl | ErrorKind::ProxyUnauthorized => 5,
        ErrorKind::InsecureRequestHttpsOnly => 1,
        _ => {
            if lower.contains("certificate") || lower.contains("tls") {
                60
            } else {
                56
            }
        }
    };
    TransportError::new(code, text)
}

// ---------------------------------------------------------------------------
// TLS verification switch (setSslVerification(false))
// ---------------------------------------------------------------------------

mod insecure {
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{DigitallySignedStruct, Error, SignatureScheme};

    /// Accepts any certificate. Only used after `set_ssl_verification(false)`.
    #[derive(Debug)]
    pub struct NoVerification(pub rustls::crypto::WebPkiSupportedAlgorithms);

    impl ServerCertVerifier for NoVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name: &ServerName<'_>,
            _ocsp_response: &[u8],
            _now: UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &CertificateDer<'_>,
            _dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            self.0.supported_schemes()
        }
    }
}

fn insecure_tls_config() -> rustls::ClientConfig {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let algorithms = provider.signature_verification_algorithms;
    rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("ring supports the default TLS versions")
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(insecure::NoVerification(algorithms)))
        .with_no_client_auth()
}
