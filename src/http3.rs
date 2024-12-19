use hickory_proto::error::ProtoError;
use hickory_proto::rr::rdata::svcb::SvcParamValue;
use hickory_proto::rr::RData;

use tokio::net::TcpStream as TokioTcpStream;
use hickory_client::client::{AsyncClient, ClientHandle};
use hickory_client::proto::iocompat::AsyncIoTokioAsStd;
use hickory_client::rr::Name;
use hickory_client::tcp::TcpClientStream;

pub struct DNSQuicProbe {
    client: AsyncClient,
    bg_join_handle: tokio::task::JoinHandle<Result<(), ProtoError>>,
}

impl DNSQuicProbe {
    pub async fn init() -> Self {
        // todo: use the DNS server from the system config
        let (stream, sender) =
            TcpClientStream::<AsyncIoTokioAsStd<TokioTcpStream>>::new(([8, 8, 8, 8], 53).into());
        let (client, bg) = AsyncClient::new(stream, sender, None).await.unwrap();

        let bg_join_handle= tokio::spawn(bg);

        DNSQuicProbe { client, bg_join_handle }
    }

    pub async fn supports_http3_dns(self: &mut Self, host: &String) -> bool {    
        let domain_name = Name::from_utf8(host).unwrap();
    
        let response = self.client.query(
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
}

impl Drop for DNSQuicProbe {
    fn drop(&mut self) {
        self.bg_join_handle.abort();
    }
}