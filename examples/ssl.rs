//! Lists your certificates and shows the purchase flow. Buying charges the balance: use OTE.
//!
//! ```bash
//! export DNA_RESELLER_ID=... DNA_API_KEY=... DNA_ENV=ote
//! cargo run --example ssl -- example.com PositiveSSL
//! ```

use domainnameapi::ssl::{
    AuthenticationType, SslApproversRequest, SslConfigureRequest, SslContact, SslGenerateCsrRequest, SslListRequest,
    WebServerType,
};
use domainnameapi::Client;
use futures_util::TryStreamExt;

#[tokio::main]
async fn main() -> Result<(), domainnameapi::Error> {
    let client = Client::from_env()?;
    let mut args = std::env::args().skip(1);
    let domain = args.next().expect("usage: ssl <domain> [package-code]");

    // Certificates are addressed by their ID; list them to find it.
    let mut certificates = client.ssl().list_all(SslListRequest::new().domain_name(&domain));
    while let Some(c) = certificates.try_next().await? {
        println!("{} {} {} until {:?}", c.certificate_id, c.package_name, c.status, c.subscription_expires_at);
    }

    let Some(package) = args.next() else { return Ok(()) };

    // Buy (charges the balance) and keep the certificate ID.
    let approvers = client.ssl().approvers(SslApproversRequest::for_purchase(&domain, &package)).await?;
    let order = client.ssl().initialize(&domain, &package).await?;
    if !order.success {
        println!("refused: {}", order.message);
        return Ok(());
    }
    println!("certificate {}", order.certificate_id);

    let csr = client
        .ssl()
        .generate_csr(
            SslGenerateCsrRequest::new(&domain)
                .country("TR")
                .state("Istanbul")
                .locality("Istanbul")
                .organization("Example Ltd")
                .organization_unit("IT"),
        )
        .await?;

    let contact = SslContact {
        first_name: "Ada".into(),
        last_name: "Lovelace".into(),
        company_name: "Example Ltd".into(),
        email: "ada@example.com".into(),
        address: "1 Example Street".into(),
        city: "Istanbul".into(),
        postal_code: "34000".into(),
        country: "TR".into(),
        phone_country_code: "90".into(),
        phone: "5551234567".into(),
        ..Default::default()
    };
    let mut configure = SslConfigureRequest::new(&order.certificate_id, &csr.csr, contact)
        .private_key(&csr.private_key)
        .authentication_type(AuthenticationType::Email)
        .web_server_type(WebServerType::Apache);
    if let Some(approver) = approvers.first() {
        configure = configure.approver_mail(approver);
    }
    let configured = client.ssl().configure(configure).await?;
    println!("configured: {} {}", configured.success, configured.dcv_details);

    let status = client.ssl().status(&order.certificate_id).await?;
    println!("issued: {}", status.is_issued);
    Ok(())
}
