use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use crate::{
    core::{ServicePacket, ServiceRequest},
    repositories::{RequestRegistry, ServiceRepository},
};

pub(crate) trait ServiceEventSink {
    fn send(&self, packet: ServicePacket) -> Result<(), String>;
}

pub(crate) struct ServiceProxy {
    repository: ServiceRepository,
    requests: RequestRegistry,
}

impl ServiceProxy {
    pub(crate) fn new(repository: ServiceRepository, requests: RequestRegistry) -> Self {
        Self {
            repository,
            requests,
        }
    }

    pub(crate) async fn execute(
        &self,
        request_id: String,
        request: ServiceRequest,
        events: &impl ServiceEventSink,
    ) -> Result<(), String> {
        if request_id.trim().is_empty() {
            return Err("A service request identifier is required.".to_owned());
        }

        let cancellation = CancellationToken::new();
        self.requests
            .register(request_id.clone(), cancellation.clone())?;
        let result = self.forward(request, events, cancellation).await;
        let cleanup = self.requests.remove(&request_id);

        match (result, cleanup) {
            (Err(error), _) => Err(error),
            (Ok(()), Err(error)) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
        }
    }

    pub(crate) fn cancel(&self, request_id: &str) -> Result<bool, String> {
        self.requests.cancel(request_id)
    }

    async fn forward(
        &self,
        request: ServiceRequest,
        events: &impl ServiceEventSink,
        cancellation: CancellationToken,
    ) -> Result<(), String> {
        let request = self.repository.build_request(&request)?;
        let response = tokio::select! {
            _ = cancellation.cancelled() => {
                return events.send(ServicePacket::error(
                    "canceled",
                    "The service request was canceled.",
                ));
            }
            response = self.repository.execute(request) => {
                match response {
                    Ok(response) => response,
                    Err(_) => {
                        return events.send(ServicePacket::error(
                            "service_unavailable",
                            "The Curio service is unavailable.",
                        ));
                    }
                }
            }
        };

        events.send(ServicePacket::Started {
            status: response.status().as_u16(),
        })?;

        let mut stream = response.bytes_stream();
        loop {
            let next = tokio::select! {
                _ = cancellation.cancelled() => {
                    return events.send(ServicePacket::error(
                        "canceled",
                        "The service request was canceled.",
                    ));
                }
                next = stream.next() => next,
            };

            match next {
                Some(Ok(bytes)) => events.send(ServicePacket::chunk(&bytes))?,
                Some(Err(_)) => {
                    return events.send(ServicePacket::error(
                        "service_unavailable",
                        "The connection to the Curio service was interrupted.",
                    ));
                }
                None => return events.send(ServicePacket::End),
            }
        }
    }
}
