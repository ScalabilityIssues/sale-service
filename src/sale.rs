use crate::error::ApplicationError;
use crate::proto::flightmngr::{Flight, SearchFlightsRequest};
use crate::proto::salesvc::sale_server::Sale;
use crate::proto::salesvc::{
    Offer, OfferClaims, PurchaseOfferRequest, PurchaseOfferResponse, SearchOffersRequest,
    SearchOffersResponse,
};
use prost_types::Timestamp;
use time::{Duration, OffsetDateTime};
use tonic::{Request, Response, Status};

use crate::dependencies::Dependencies;
use crate::tokens::TagManager;

pub struct SaleApp {
    deps: Dependencies,
    token_manager: TagManager,
}

impl From<SearchOffersRequest> for SearchFlightsRequest {
    fn from(value: SearchOffersRequest) -> Self {
        Self {
            origin_id: value.departure_airport,
            destination_id: value.arrival_airport,
            departure_day: value.departure_date,
        }
    }
}

async fn create_offer(
    flight: Flight,
    deps: &Dependencies,
    expiration: i64,
    tm: &TagManager,
) -> Result<Offer, ApplicationError> {
    let price = deps.get_price_estimation(flight.clone()).await?;
    Ok(Offer {
        tag: tm.generate_tag(&flight.id, &price, expiration),
        flight: Some(flight),
        price: Some(price),
        expiration: Some(Timestamp {
            seconds: expiration,
            nanos: 0,
        }),
    })
}

#[tonic::async_trait]
impl Sale for SaleApp {
    async fn search_offers(
        &self,
        request: Request<SearchOffersRequest>,
    ) -> Result<Response<SearchOffersResponse>, Status> {
        let r = request.into_inner();

        let f = self.deps.search_flights(r.into()).await?;

        let exp = (OffsetDateTime::now_utc() + Duration::minutes(15)).unix_timestamp();

        let offers = futures_util::future::join_all(
            f.into_iter()
                .map(|f| create_offer(f, &self.deps, exp, &self.token_manager)),
        )
        .await
        .into_iter()
        .collect::<Result<_, _>>()?;

        Ok(Response::new(SearchOffersResponse { offers }))
    }

    async fn purchase_offer(
        &self,
        request: Request<PurchaseOfferRequest>,
    ) -> Result<Response<PurchaseOfferResponse>, Status> {
        let PurchaseOfferRequest {
            data: Some(user_data),
            offer:
                Some(OfferClaims {
                    flight_id,
                    price: Some(price),
                    expiration:
                        Some(Timestamp {
                            seconds: expiration,
                            ..
                        }),
                }),
            tag,
        } = request.into_inner()
        else {
            return Err(Status::invalid_argument("invalid request"));
        };

        self.token_manager
            .verify_offer(&flight_id, &price, expiration, &tag)?;

        let ticket = self.deps.create_ticket(flight_id, user_data).await?;

        Ok(Response::new(PurchaseOfferResponse {
            ticket: Some(ticket),
        }))
    }
}

impl SaleApp {
    pub fn new(deps: Dependencies, token_manager: TagManager) -> Self {
        Self {
            deps,
            token_manager,
        }
    }
}
