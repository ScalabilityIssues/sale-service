use crate::proto::salesvc::sale_server::Sale;
use crate::proto::salesvc::{Offer, SearchOffersRequest, SearchOffersResponse};
use tonic::{Request, Response, Status};

use crate::dependencies::Dependencies;

pub struct SaleApp {
    deps: Dependencies,
}

#[tonic::async_trait]
impl Sale for SaleApp {
    async fn search_offers(
        &self,
        _request: Request<SearchOffersRequest>,
    ) -> Result<Response<SearchOffersResponse>, Status> {
        let f = self.deps.list_flights().await?;

        let offers = f
            .into_iter()
            .map(|f| Offer {
                flight: Some(f),
                price: Default::default(),
                token: "token".to_string(),
            })
            .collect();

        Ok(Response::new(SearchOffersResponse { offers }))
    }
}

impl SaleApp {
    pub fn new(deps: Dependencies) -> Self {
        Self { deps }
    }
}
