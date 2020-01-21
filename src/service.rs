use tonic::{
    transport::Server,
    {Request, Response, Status},
};

pub mod ruft_rpc {
    tonic::include_proto!("ruft_rpc");
}

use ruft_rpc::{
    rpc_client::RpcClient,
    rpc_server::{Rpc, RpcServer},
    AppendEntriesReq, AppendEntriesResp, RequestAVoteReq, RequestAVoteResp,
};

#[derive(Debug, Default)]
pub struct RPCService {}

#[tonic::async_trait]
impl Rpc for RPCService {
    async fn append_entries(
        &self,
        request: Request<AppendEntriesReq>, // Accept request of type AppendEntriesReq
    ) -> Result<Response<AppendEntriesResp>, Status> {
        // Return an instance of type AppendEntriesResp
        println!("append_entries got a request: {:?}", request);

        let reply = AppendEntriesResp {
            term: 42,
            success: true,
        };

        Ok(Response::new(reply))
    }

    async fn request_a_vote(
        &self,
        request: Request<RequestAVoteReq>,
    ) -> Result<Response<RequestAVoteResp>, Status> {
        println!("request_a_vote got a request: {:?}", request);

        let reply = RequestAVoteResp {
            term: 42,
            vote_granted: true,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
pub async fn init_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let host_port = format!("[::1]:{}", port);
    let addr = host_port.parse()?;
    let rpc_service = RPCService::default();

    Server::builder()
        .add_service(RpcServer::new(rpc_service))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
pub async fn append_entries(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let host_port = format!("http://[::1]:{}", port);
    let mut client = RpcClient::connect(host_port).await?;

    let request = tonic::Request::new(AppendEntriesReq { term: 42 });

    let response = client.append_entries(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

#[tokio::main]
pub async fn request_a_vote(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let host_port = format!("http://[::1]:{}", port);
    let mut client = RpcClient::connect(host_port).await?;

    let request = tonic::Request::new(RequestAVoteReq { term: 42 });

    let response = client.request_a_vote(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
