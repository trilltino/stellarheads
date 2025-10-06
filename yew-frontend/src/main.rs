use yew::prelude::*;
use yew::Renderer;
use yew_router::prelude::*;

mod components;
mod pages;
mod routing;
mod services;
mod wallet;

use routing::{Route, switch};

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