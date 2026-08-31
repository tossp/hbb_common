use base64::{engine::general_purpose::STANDARD, Engine as _};

fn tossp_client_defaults() -> Result<(), &'static str> {
    let rendezvous_server = std::env::var("RENDEZVOUS_SERVER")
        .map_err(|_| "tossp-client requires a valid RENDEZVOUS_SERVER")?;
    if !is_fqdn(&rendezvous_server) {
        return Err("tossp-client requires a valid RENDEZVOUS_SERVER");
    }

    let rs_pub_key = std::env::var("RS_PUB_KEY")
        .map_err(|_| "tossp-client requires a valid RS_PUB_KEY")?;
    let decoded = STANDARD
        .decode(rs_pub_key.as_bytes())
        .map_err(|_| "tossp-client requires a valid RS_PUB_KEY")?;
    if decoded.len() != 32 || STANDARD.encode(&decoded) != rs_pub_key {
        return Err("tossp-client requires a valid RS_PUB_KEY");
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set by Cargo");
    std::fs::write(
        std::path::Path::new(&out_dir).join("tossp_client_defaults.rs"),
        format!(
            "pub const TOSSP_RENDEZVOUS_SERVER: &str = {rendezvous_server:?};\n\
             pub const TOSSP_RS_PUB_KEY: &str = {rs_pub_key:?};\n"
        ),
    )
    .expect("failed to write TossPig client defaults");
    Ok(())
}

fn is_fqdn(value: &str) -> bool {
    if value.len() > 253
        || !value.contains('.')
        || value.parse::<std::net::IpAddr>().is_ok()
        || value.bytes().any(|byte| byte.is_ascii_uppercase())
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return false;
    }

    value.split('.').all(|label| {
        let bytes = label.as_bytes();
        !bytes.is_empty()
            && bytes.len() <= 63
            && bytes[0].is_ascii_alphanumeric()
            && bytes[bytes.len() - 1].is_ascii_alphanumeric()
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    })
}

fn main() {
    if std::env::var_os("CARGO_FEATURE_TOSSP_CLIENT").is_some() {
        println!("cargo:rerun-if-env-changed=RENDEZVOUS_SERVER");
        println!("cargo:rerun-if-env-changed=RS_PUB_KEY");
        tossp_client_defaults().unwrap_or_else(|error| panic!("{error}"));
    }

    let out_dir = format!("{}/protos", std::env::var("OUT_DIR").unwrap());

    std::fs::create_dir_all(&out_dir).unwrap();

    protobuf_codegen::Codegen::new()
        .pure()
        .out_dir(out_dir)
        .inputs(["protos/rendezvous.proto", "protos/message.proto"])
        .include("protos")
        .customize(protobuf_codegen::Customize::default().tokio_bytes(true))
        .run()
        .expect("Codegen failed.");
}
