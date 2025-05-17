use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path::end().map(|| "Hello, HTTPS!\n");

    let cert_path = "cert/cert.pem";
    let key_path = "cert/key.pem";

    warp::serve(hello)
        .tls()
        .cert_path(cert_path)
        .key_path(key_path)
        .run(([127, 0, 0, 1], 8443))
        .await;
}