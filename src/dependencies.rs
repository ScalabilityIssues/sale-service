use crate::error::Result;
use crate::proto::flightmngr::{self, flights_client::FlightsClient, ListFlightsRequest};
use tonic::transport::Channel;

#[derive(Clone, Debug)]
pub struct Dependencies {
    pub flights: FlightsClient<Channel>,
}

impl Dependencies {
    pub fn new(flightmngr_channel: Channel) -> Self {
        Self {
            flights: FlightsClient::new(flightmngr_channel),
        }
    }

    pub async fn list_flights(&self) -> Result<Vec<flightmngr::Flight>> {
        let r = self
            .flights
            .clone()
            .list_flights(ListFlightsRequest {
                include_cancelled: true,
            })
            .await?;

        Ok(r.into_inner().flights)
    }
}
