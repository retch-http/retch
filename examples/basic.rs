use retch::retcher::Retcher;
use retch::emulation::Browser;
 
 #[tokio::main]
 async fn main() {
    let mut retcher = Retcher::builder()
        .with_browser(Browser::Firefox)
        .with_http3()
        .build();

    let response = retcher.get(String::from("https://example.com"), None).await;

    match response {
        Ok(response) => {
            println!("{}", response.text().await.unwrap());
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
 }