use yew::Renderer;

mod api;
mod freighter;
mod homepage;
mod loginpage;
mod gamepage;
mod navbar;
mod routes;
mod soroban;


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