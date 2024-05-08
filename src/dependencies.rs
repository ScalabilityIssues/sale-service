use tonic::transport::Channel;

use crate::config::DependencyConfig;
use crate::error::{ApplicationError, Result};
use crate::proto::flightmngr::Flight;
use crate::proto::flightmngr::{self, flights_client::FlightsClient};
use crate::proto::google::r#type::Money;
use crate::proto::priceest::price_estimation_client::PriceEstimationClient;
use crate::proto::priceest::{EstimatePriceRequest, FlightDetails};

#[derive(Clone, Debug)]
pub struct Dependencies {
    pub flights: FlightsClient<Channel>,
    pub priceest: PriceEstimationClient<Channel>,
}

impl From<Flight> for FlightDetails {
    fn from(value: Flight) -> Self {
        Self {
            destination: value.destination_id,
            source: value.origin_id,
            departure_time: value.departure_time,
            arrival_time: value.arrival_time,
        }
    }
}

impl Dependencies {
    pub fn new(
        dependency_urls: DependencyConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let DependencyConfig {
            flightmngr_url,
            priceest_url,
            ..
        } = dependency_urls;

        let flightmngr_channel = Channel::builder(flightmngr_url.try_into()?).connect_lazy();
        let priceest_channel = Channel::builder(priceest_url.try_into()?).connect_lazy();

        Ok(Self {
            flights: FlightsClient::new(flightmngr_channel),
            priceest: PriceEstimationClient::new(priceest_channel),
        })
    }

    pub async fn search_flights(
        &self,
        request: flightmngr::SearchFlightsRequest,
    ) -> Result<Vec<flightmngr::Flight>> {
        let r = self.flights.clone().search_flights(request).await?;
        Ok(r.into_inner().flights)
    }

    pub async fn get_price_estimation(&self, flight: flightmngr::Flight) -> Result<Money> {
        let request = EstimatePriceRequest {
            flight: Some(flight.into()),
        };
        let r = self.priceest.clone().estimate_price(request).await?;

        r.into_inner()
            .price
            .ok_or(ApplicationError::unexpected_error(
                "no price found in response from priceestimator",
            ))
    }
}
