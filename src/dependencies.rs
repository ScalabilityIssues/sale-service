use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use prost_types::Timestamp;
use time::OffsetDateTime;
use tonic::transport::Channel;

use crate::config::DependencyConfig;
use crate::error::{ApplicationError, Result};
use crate::proto::flightmngr::airports_client::AirportsClient;
use crate::proto::flightmngr::ListAirportsRequest;
use crate::proto::flightmngr::{self, flights_client::FlightsClient};
use crate::proto::google::r#type::Money;
use crate::proto::priceest::price_estimation_client::PriceEstimationClient;
use crate::proto::priceest::{EstimatePriceRequest, FlightDetails};
use crate::proto::ticketsrvc::tickets_client::TicketsClient;
use crate::proto::ticketsrvc::{CreateTicketRequest, PassengerDetails, Ticket};

#[derive(Clone, Debug)]
pub struct Dependencies {
    airports: AirportsClient<Channel>,
    flights: FlightsClient<Channel>,
    priceest: PriceEstimationClient<Channel>,
    tickets: TicketsClient<Channel>,

    airport_iata_map: Arc<RwLock<HashMap<String, String>>>,

    fake_price: bool,
}

impl Dependencies {
    pub fn new(
        dependency_urls: DependencyConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let DependencyConfig {
            flightmngr_url,
            priceest_url,
            ticketsvc_url,
            fake_price,
        } = dependency_urls;

        let flightmngr_channel = Channel::builder(flightmngr_url.try_into()?).connect_lazy();
        let priceest_channel = Channel::builder(priceest_url.try_into()?).connect_lazy();
        let tickets_channel = Channel::builder(ticketsvc_url.try_into()?).connect_lazy();

        Ok(Self {
            airports: AirportsClient::new(flightmngr_channel.clone()),
            flights: FlightsClient::new(flightmngr_channel),
            priceest: PriceEstimationClient::new(priceest_channel),
            tickets: TicketsClient::new(tickets_channel),
            fake_price,
            airport_iata_map: Default::default(),
        })
    }

    async fn get_airport_iata(&self, airport_id: &str) -> Result<String> {
        if let Some(iata) = self.airport_iata_map.read().unwrap().get(airport_id) {
            return Ok(iata.clone());
        }

        let r = self
            .airports
            .clone()
            .list_airports(ListAirportsRequest { show_deleted: true })
            .await?
            .into_inner();

        for airport in r.airports {
            self.airport_iata_map
                .write()
                .unwrap()
                .insert(airport.id.clone(), airport.iata.clone());
        }

        self.airport_iata_map
            .read()
            .unwrap()
            .get(airport_id)
            .cloned()
            .ok_or(ApplicationError::unexpected_error(
                "airport not found in response from airports service",
            ))
    }

    pub async fn search_flights(
        &self,
        request: flightmngr::SearchFlightsRequest,
    ) -> Result<Vec<flightmngr::Flight>> {
        let r = self.flights.clone().search_flights(request).await?;
        Ok(r.into_inner().flights)
    }

    pub async fn get_price_estimation(&self, flight: flightmngr::Flight) -> Result<Money> {
        if self.fake_price {
            return Ok(Money {
                currency_code: "USD".to_string(),
                units: 100,
                nanos: 0,
            });
        }

        let request = EstimatePriceRequest {
            flight: Some(FlightDetails {
                destination: self.get_airport_iata(&flight.destination_id).await?,
                source: self.get_airport_iata(&flight.origin_id).await?,
                departure_time: flight.departure_time,
                arrival_time: flight.arrival_time,
            }),
        };
        let r = self.priceest.clone().estimate_price(request).await?;

        r.into_inner()
            .price
            .ok_or(ApplicationError::unexpected_error(
                "no price found in response from priceestimator",
            ))
    }

    pub async fn create_ticket(
        &self,
        flight_id: String,
        passenger: PassengerDetails,
    ) -> Result<Ticket> {
        let now = OffsetDateTime::now_utc();

        let ticket = Some(Ticket {
            flight_id,
            passenger: Some(passenger),
            reservation_datetime: Some(Timestamp {
                seconds: now.unix_timestamp(),
                nanos: now.nanosecond() as i32,
            }),
            ..Default::default()
        });

        let r = self
            .tickets
            .clone()
            .create_ticket(CreateTicketRequest { ticket })
            .await?;

        Ok(r.into_inner())
    }
}
