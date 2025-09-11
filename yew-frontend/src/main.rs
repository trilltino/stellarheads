use yew::Renderer;

mod freighter;
mod loginpage;
mod gamepage;
mod routes;
mod soroban;
mod game_score;
mod contract_test;

use routes::{Route, switch};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    Renderer::<App>::new().render();
}