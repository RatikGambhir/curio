use tauri::ipc::Channel;

use crate::{core::ServicePacket, services::ServiceEventSink};

pub(crate) struct ServiceChannel(Channel<ServicePacket>);

impl ServiceChannel {
    pub(crate) fn new(channel: Channel<ServicePacket>) -> Self {
        Self(channel)
    }
}

impl ServiceEventSink for ServiceChannel {
    fn send(&self, packet: ServicePacket) -> Result<(), String> {
        self.0
            .send(packet)
            .map_err(|_| "Desktop could not deliver a service response packet.".to_owned())
    }
}
