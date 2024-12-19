use hickory_proto::rr::rdata::svcb::SvcParamValue;
use hickory_proto::rr::RData;
use url::Url;

use tokio::net::TcpStream as TokioTcpStream;
use hickory_client::client::{AsyncClient, ClientHandle};
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_client::rr::Name;
use hickory_client::tcp::TcpClientStream;

pub async fn supports_http3_dns(url: Url) -> bool {
    // todo: use the DNS server from the system config
    let (stream, sender) =
    TcpClientStream::<AsyncIoTokioAsStd<TokioTcpStream>>::new(([8, 8, 8, 8], 53).into());
    let (mut client, bg) = AsyncClient::new(stream, sender, None).await.unwrap();

    tokio::spawn(bg);

    let domain_name = Name::from_utf8(url.host_str().unwrap()).unwrap();

    let response = client.query(
        domain_name,
        hickory_proto::rr::DNSClass::IN, 
        hickory_proto::rr::RecordType::HTTPS
    ).await;

    response.is_ok_and(|response | {
        response.answers().iter().any(|answer| {
            if let RData::HTTPS(data) = answer.data().unwrap() {
                return data.svc_params().iter().any(|param| {
                    if let SvcParamValue::Alpn(alpn_protocols) = param.1.clone() {
                        return alpn_protocols.0.iter().any(|alpn| { 
                            alpn == "h3" 
                        })
                    }

                    false
                });
            }
            false
        })
    })
}