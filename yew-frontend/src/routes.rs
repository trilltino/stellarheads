use crate::gamepage::GamePage;
use crate::navbar::Navbar;
use crate::homepage::HomePage;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,

    #[at("/about")]
    About,

    #[at("/learn-more")]
    LearnMore,

    #[at("/game")]
    Game,

    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(HomePageWithNav)]
fn home_page_with_nav() -> Html {
    html! {
        <div class="page-layout">
            <Navbar />
            <div class="page-content">
                <HomePage />
            </div>
        </div>
    }
}


#[function_component(AboutPage)]
fn about_page() -> Html {
    html! {
        <div class="page-layout">
            <Navbar />
            <div class="page-content">
                <div class="about-container">
                    <h1>{"About Stellar Heads"}</h1>
                    <div class="about-content">
                        <p>{"Stellar Heads is an innovative blockchain-powered soccer game built on the Stellar network. Experience the thrill of competitive gameplay while earning rewards on one of the most efficient and eco-friendly blockchain platforms."}</p>

                        <h2>{"Our Mission"}</h2>
                        <p>{"We're revolutionizing the gaming industry by combining traditional gameplay with blockchain technology, creating a seamless experience where players can compete, earn, and own their in-game achievements."}</p>

                        <h2>{"Why Stellar?"}</h2>
                        <p>{"Stellar provides fast, low-cost transactions and a robust ecosystem for building decentralized applications. Our choice of Stellar ensures that players can enjoy instant gameplay without high transaction fees."}</p>

                        <h2>{"Key Features"}</h2>
                        <ul>
                            <li>{"Real-time multiplayer gameplay"}</li>
                            <li>{"Blockchain-based leaderboards"}</li>
                            <li>{"Instant rewards and payments"}</li>
                            <li>{"Cross-platform compatibility"}</li>
                            <li>{"Eco-friendly blockchain technology"}</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[function_component(LearnMorePage)]
fn learn_more_page() -> Html {
    html! {
        <div class="page-layout">
            <Navbar />
            <div class="page-content">
                <div class="learn-more-container">
                    <h1>{"Learn More About Stellar Heads"}</h1>

                    <div class="learn-section">
                        <h2>{"How to Play"}</h2>
                        <div class="game-instructions">
                            <p>{"Stellar Heads is a fast-paced soccer game where you control a player to score goals against AI opponents or other players."}</p>
                            <h3>{"Controls:"}</h3>
                            <ul>
                                <li><strong>{"Arrow Keys / WASD:"}</strong> {" Move your player"}</li>
                                <li><strong>{"Spacebar:"}</strong> {" Jump"}</li>
                                <li><strong>{"R:"}</strong> {" Reset the match"}</li>
                            </ul>
                        </div>
                    </div>

                    <div class="learn-section">
                        <h2>{"Blockchain Integration"}</h2>
                        <p>{"Connect your Freighter wallet to:"}</p>
                        <ul>
                            <li>{"Track your game statistics on-chain"}</li>
                            <li>{"Compete on global leaderboards"}</li>
                            <li>{"Earn rewards for your performance"}</li>
                            <li>{"Participate in tournaments"}</li>
                        </ul>
                    </div>

                    <div class="learn-section">
                        <h2>{"Getting Started"}</h2>
                        <ol>
                            <li>{"Install the Freighter wallet browser extension"}</li>
                            <li>{"Create or import a Stellar account"}</li>
                            <li>{"Connect your wallet on the home page"}</li>
                            <li>{"Start playing and earning!"}</li>
                        </ol>
                    </div>

                    <div class="learn-section">
                        <h2>{"Technical Stack"}</h2>
                        <p>{"Built with cutting-edge technology:"}</p>
                        <ul>
                            <li><strong>{"Bevy Engine:"}</strong> {" High-performance game engine written in Rust"}</li>
                            <li><strong>{"WebAssembly:"}</strong> {" Near-native performance in the browser"}</li>
                            <li><strong>{"Yew Framework:"}</strong> {" Modern web frontend framework"}</li>
                            <li><strong>{"Stellar Network:"}</strong> {" Fast and efficient blockchain"}</li>
                            <li><strong>{"Avian2D:"}</strong> {" Physics simulation"}</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[function_component(GamePageWithNav)]
fn game_page_with_nav() -> Html {
    html! {
        <div class="page-layout">
            <Navbar />
            <div class="page-content">
                <GamePage />
            </div>
        </div>
    }
}


pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePageWithNav /> },
        Route::About => html! { <AboutPage /> },
        Route::LearnMore => html! { <LearnMorePage /> },
        Route::Game => html! { <GamePageWithNav /> },
        Route::NotFound => html! {
            <div class="page-layout">
                <Navbar />
                <div class="page-content">
                    <div class="not-found">
                        <h1>{"404 - Page Not Found"}</h1>
                        <p>{"The page you're looking for doesn't exist."}</p>
                        <a href="/">{"Go Home"}</a>
                    </div>
                </div>
            </div>
        },
    }
}



