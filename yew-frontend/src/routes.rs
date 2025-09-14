use crate::loginpage::LoginPage;
use crate::gamepage::GamePage;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/setup")]
    Setup,

    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <LoginPage /> },
        Route::Setup => html! { <GamePage /> },
        Route::NotFound => html! {
            <div class="not-found">
                <h1>{"404 - Page Not Found"}</h1>
                <p>{"The page you're looking for doesn't exist."}</p>
                <a href="/">{"Go Home"}</a>
            </div>
        },
    }
}



