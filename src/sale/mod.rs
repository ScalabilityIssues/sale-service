use crate::error::ApplicationError;
use crate::proto::flightmngr::{Flight, SearchFlightsRequest};
use crate::proto::salesvc::sale_server::Sale;
use crate::proto::salesvc::{
    Offer, PurchaseOfferRequest, PurchaseOfferResponse, SearchOffersRequest,
    SearchOffersResponse,
};
use tonic::{Request, Response, Status};

use crate::dependencies::Dependencies;

pub struct SaleApp {
    deps: Dependencies,
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

async fn create_offer(flight: Flight, deps: Dependencies) -> Result<Offer, ApplicationError> {
    let price = deps.get_price_estimation(flight.clone()).await?;
    Ok(Offer {
        flight: Some(flight),
        price: Some(price),
        token: "token".to_string(), // TODO
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

        let offers = futures_util::future::join_all(
            f.into_iter().map(|f| create_offer(f, self.deps.clone())),
        )
        .await
        .into_iter()
        .collect::<Result<_, _>>()?;

        Ok(Response::new(SearchOffersResponse { offers }))
    }

    async fn purchase_offer(
        &self,
        _request: Request<PurchaseOfferRequest>,
    ) -> Result<Response<PurchaseOfferResponse>, Status> {
        todo!()
    }
}

impl SaleApp {
    pub fn new(deps: Dependencies) -> Self {
        Self { deps }
    }
}
