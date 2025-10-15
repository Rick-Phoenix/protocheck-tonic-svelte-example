use std::{net::SocketAddr, time::Duration};

use axum::{body::Bytes, routing::get, Router};
use greeter::farewell_server::Farewell;
use http::{header, HeaderName, Method, StatusCode};
use prost::Message;
use proto_types::Status as GrpcStatus;
use protocheck::ProtoValidator;
use tonic::{Code, Request as TonicRequest, Response as TonicResponse, Status};
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub mod greeter {
  tonic::include_proto!("greeter");

  pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("tonic_descriptor");
}

use crate::greeter::{
  farewell_server::FarewellServer,
  greeter_server::{Greeter, GreeterServer},
  HelloReply, HelloRequest,
};

#[derive(Default, Clone)]
pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
  async fn say_hello(
    &self,
    request: TonicRequest<HelloRequest>,
  ) -> Result<TonicResponse<HelloReply>, Status> {
    validate(request, |req| async move {
      let reply = HelloReply {
        message: format!("Hello, {}!", req.into_inner().name),
      };

      Ok(TonicResponse::new(reply))
    })
    .await
  }
}

async fn root_handler() -> &'static str {
  "Hello from Axum! This is the REST API."
}

async fn health_check_handler() -> &'static str {
  "OK"
}

#[derive(Default, Clone)]
pub struct MyFarewell;

#[tonic::async_trait]
impl Farewell for MyFarewell {
  async fn say_good_bye(
    &self,
    request: tonic::Request<HelloRequest>,
  ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
    let reply = HelloReply {
      message: format!("Goodbye, {}!", request.into_inner().name),
    };
    Ok(tonic::Response::new(reply))
  }
}

pub async fn validate<T, F, Fut, R>(
  req: TonicRequest<T>,
  handler: F,
) -> Result<TonicResponse<R>, Status>
where
  T: ProtoValidator,
  F: FnOnce(TonicRequest<T>) -> Fut,
  Fut: Future<Output = Result<TonicResponse<R>, Status>>,
{
  if let Err(violations) = req.get_ref().validate() {
    let status_inner: GrpcStatus = violations.into();

    let status = Status::with_details(
      Code::InvalidArgument,
      "Validation Error",
      Bytes::from(status_inner.encode_to_vec()),
    );

    return Err(status);
  }

  handler(req).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  println!("🚀 Server listening on http://{}", addr);

  let axum_router = Router::new()
    .route("/", get(root_handler))
    .route("/health", get(health_check_handler));

  let cors = CorsLayer::new()
    .allow_origin(AllowOrigin::mirror_request()) // For development only
    .allow_credentials(true)
    .allow_methods([Method::POST])
    // Allow incoming headers
    .allow_headers([
      header::CONTENT_TYPE,
      header::AUTHORIZATION,
      HeaderName::from_static("x-grpc-web"),
      HeaderName::from_static("x-user-agent"),
      HeaderName::from_static("grpc-timeout"),
    ])
    .max_age(Duration::from_secs(24 * 60 * 60))
    // Allow client's js to read these
    .expose_headers([
      HeaderName::from_static("grpc-status"),
      HeaderName::from_static("grpc-message"),
      HeaderName::from_static("grpc-status-details-bin"),
    ]);

  let reflection_service = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(greeter::FILE_DESCRIPTOR_SET)
    .build_v1()
    .unwrap();

  let mut routes_builder = tonic::service::Routes::builder();

  routes_builder
    .add_service(GreeterServer::new(MyGreeter::default()))
    .add_service(FarewellServer::new(MyFarewell::default()))
    .add_service(reflection_service);

  let grpc_router = routes_builder
    .routes()
    .into_axum_router()
    .layer(GrpcWebLayer::new())
    .layer(cors);

  let app = axum_router.merge(grpc_router).fallback(fallback);

  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}

async fn fallback() -> StatusCode {
  StatusCode::NOT_FOUND
}
