pub mod tokens;

use crate::error::ApplicationError;
use crate::proto::flightmngr::{Flight, SearchFlightsRequest};
use crate::proto::salesvc::sale_server::Sale;
use crate::proto::salesvc::{
    Offer, PurchaseOfferRequest, PurchaseOfferResponse, SearchOffersRequest, SearchOffersResponse,
    TokenData,
};
use prost_types::Timestamp;
use time::{Duration, OffsetDateTime};
use tonic::{Request, Response, Status};

use crate::dependencies::Dependencies;

use self::tokens::TokenManager;

pub struct SaleApp {
    deps: Dependencies,
    token_manager: TokenManager,
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
    deps: Dependencies,
    expiration: i64,
    tm: &tokens::TokenManager,
) -> Result<Offer, ApplicationError> {
    let price = deps.get_price_estimation(flight.clone()).await?;
    Ok(Offer {
        token: Some(tm.generate_token(TokenData {
            flight_id: flight.id.clone(),
            price: Some(price.clone()),
            expiration: Some(Timestamp {
                seconds: expiration,
                nanos: 0,
            }),
        })),
        flight: Some(flight),
        price: Some(price),
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
                .map(|f| create_offer(f, self.deps.clone(), exp, &self.token_manager)),
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
            offer: Some(Offer {
                token: Some(token), ..
            }),
            data: Some(user_data),
        } = request.into_inner()
        else {
            return Err(Status::invalid_argument("invalid request"));
        };

        let TokenData { flight_id, .. } = self.token_manager.verify_token(token)?;

        let ticket = self.deps.create_ticket(flight_id, user_data).await?;

        Ok(Response::new(PurchaseOfferResponse {
            ticket: Some(ticket),
        }))
    }
}

impl SaleApp {
    pub fn new(deps: Dependencies, token_manager: TokenManager) -> Self {
        Self {
            deps,
            token_manager,
        }
    }
}
