use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    html! {
        <nav class="navbar">
            <div class="nav-container">
                <div class="nav-left">
                    <Link<Route> to={Route::Home} classes="nav-brand">
                        {"Stellar Heads"}
                    </Link<Route>>
                </div>

                <div class="nav-center">
                    <div class="nav-links">
                        <Link<Route> to={Route::Home} classes="nav-link">
                            {"Home"}
                        </Link<Route>>
                        <Link<Route> to={Route::About} classes="nav-link">
                            {"About"}
                        </Link<Route>>
                        <Link<Route> to={Route::LearnMore} classes="nav-link">
                            {"Learn More"}
                        </Link<Route>>
                        <Link<Route> to={Route::Game} classes="nav-link">
                            {"Play Game"}
                        </Link<Route>>
                    </div>
                </div>

                <div class="nav-right">
                    <div class="nav-social">
                        <a href="https://stellar.org" target="_blank" class="social-link">
                            <i class="fab fa-twitter"></i>
                        </a>
                        <a href="https://github.com" target="_blank" class="social-link">
                            <i class="fab fa-github"></i>
                        </a>
                    </div>
                </div>
            </div>

            <style>
                {r#"
                .navbar {
                    background-color: black;
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    z-index: 1000;
                    border-bottom: 2px solid #333;
                    padding: 0;
                    height: 70px;
                }

                .nav-container {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    max-width: 1200px;
                    margin: 0 auto;
                    height: 100%;
                    padding: 0 20px;
                }

                .nav-left {
                    flex: 1;
                }

                .nav-brand {
                    font-size: 1.8rem;
                    font-weight: bold;
                    color: #00d4ff;
                    text-decoration: none;
                    transition: color 0.3s ease;
                }

                .nav-brand:hover {
                    color: #00b4d8;
                }

                .nav-center {
                    flex: 2;
                    display: flex;
                    justify-content: center;
                }

                .nav-links {
                    display: flex;
                    gap: 40px;
                    align-items: center;
                }

                .nav-link {
                    color: white;
                    text-decoration: none;
                    font-size: 1.1rem;
                    font-weight: 500;
                    padding: 10px 15px;
                    border-radius: 5px;
                    transition: all 0.3s ease;
                    position: relative;
                }

                .nav-link:hover {
                    color: #00d4ff;
                    background-color: rgba(0, 212, 255, 0.1);
                }

                .nav-link.active {
                    color: #00d4ff;
                    background-color: rgba(0, 212, 255, 0.15);
                }

                .nav-right {
                    flex: 1;
                    display: flex;
                    justify-content: flex-end;
                }

                .nav-social {
                    display: flex;
                    gap: 15px;
                    align-items: center;
                }

                .social-link {
                    color: #888;
                    font-size: 1.2rem;
                    transition: color 0.3s ease;
                    padding: 5px;
                }

                .social-link:hover {
                    color: #00d4ff;
                }

                /* Mobile Responsive */
                @media (max-width: 768px) {
                    .nav-container {
                        padding: 0 15px;
                    }

                    .nav-links {
                        gap: 20px;
                    }

                    .nav-link {
                        font-size: 0.9rem;
                        padding: 8px 12px;
                    }

                    .nav-brand {
                        font-size: 1.5rem;
                    }
                }

                @media (max-width: 576px) {
                    .nav-links {
                        gap: 10px;
                    }

                    .nav-link {
                        font-size: 0.8rem;
                        padding: 6px 8px;
                    }

                    .nav-social {
                        gap: 10px;
                    }
                }
                "#}
            </style>
        </nav>
    }
}