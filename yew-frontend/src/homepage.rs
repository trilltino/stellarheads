use yew::prelude::*;
use yew_router::prelude::*;
use crate::routes::Route;
use crate::loginpage::LoginPage;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let show_login = use_state(|| false);
    let on_get_started = {
        let show_login = show_login.clone();
        Callback::from(move |_| {
            show_login.set(true);
        })
    };
    
    let on_close_login = {
        let show_login = show_login.clone();
        Callback::from(move |_| {
            show_login.set(false);
        })
    };
    html! {
        <>
            {
                if *show_login {
                    html! {
                        <div class="login-modal" onclick={on_close_login.clone()}>
                            <div class="login-modal-content" onclick={|e: MouseEvent| e.stop_propagation()}>
                                <button class="close-btn" onclick={on_close_login}>
                                    {"×"}
                                </button>
                                <LoginPage />
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            <section class="hero-section">
                <div class="hero-container">
                    <div class="hero-content">
                        <div class="hero-text">
                            <h1 class="hero-title">
                                {"Play "}
                                <span class="highlight">{"Stellar Heads"}</span>
                                <br />
                                {"On The Blockchain"}
                            </h1>
                            <p class="hero-description">
                                {"Experience the future of gaming with our blockchain-powered soccer game. "}
                                {"Compete, earn rewards, and climb the leaderboards on the Stellar network."}
                            </p>
                            <div class="hero-buttons">
                                <button class="cta-button primary" onclick={on_get_started.clone()}>
                                    <span>{"🚀 Get Started"}</span>
                                </button>
                                <Link<Route> to={Route::LearnMore} classes="cta-button secondary">
                                    <span>{"📖 Learn More"}</span>
                                </Link<Route>>
                            </div>
                        </div>
                        <div class="hero-visual">
                            <div class="game-preview">
                                <div class="game-screen">
                                    <div class="field">
                                        <div class="player player-1"></div>
                                        <div class="player player-2"></div>
                                        <div class="ball"></div>
                                        <div class="goal goal-left"></div>
                                        <div class="goal goal-right"></div>
                                    </div>
                                </div>
                                <div class="floating-icons">
                                    <div class="icon icon-1">{"⚽"}</div>
                                    <div class="icon icon-2">{"🏆"}</div>
                                    <div class="icon icon-3">{"💎"}</div>
                                    <div class="icon icon-4">{"🌟"}</div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </section>


            <section class="features-section">
                <div class="features-container">
                    <h2 class="section-title">{"Why Choose Stellar Heads?"}</h2>
                    <div class="features-grid">
                        <div class="feature-card">
                            <div class="feature-icon">{"⚡"}</div>
                            <h3>{"Lightning Fast"}</h3>
                            <p>{"Built with Rust and WebAssembly for near-native performance in your browser."}</p>
                        </div>
                        <div class="feature-card">
                            <div class="feature-icon">{"🌐"}</div>
                            <h3>{"Blockchain Powered"}</h3>
                            <p>{"Secure leaderboards and rewards powered by the Stellar network."}</p>
                        </div>
                        <div class="feature-card">
                            <div class="feature-icon">{"💰"}</div>
                            <h3>{"Earn Rewards"}</h3>
                            <p>{"Win matches and climb leaderboards to earn real rewards on-chain."}</p>
                        </div>
                        <div class="feature-card">
                            <div class="feature-icon">{"🎮"}</div>
                            <h3>{"Easy to Play"}</h3>
                            <p>{"Simple controls, addictive gameplay. Master the physics and dominate!"}</p>
                        </div>
                    </div>
                </div>
            </section>


            <section class="stats-section">
                <div class="stats-container">
                    <div class="stats-grid">
                        <div class="stat-item">
                            <div class="stat-number">{"1,000+"}</div>
                            <div class="stat-label">{"Matches Played"}</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-number">{"50+"}</div>
                            <div class="stat-label">{"Active Players"}</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-number">{"100%"}</div>
                            <div class="stat-label">{"On-Chain Verified"}</div>
                        </div>
                        <div class="stat-item">
                            <div class="stat-number">{"<1s"}</div>
                            <div class="stat-label">{"Transaction Speed"}</div>
                        </div>
                    </div>
                </div>
            </section>


            <section class="cta-section">
                <div class="cta-container">
                    <h2>{"Ready to Start Playing?"}</h2>
                    <p>{"Connect your Freighter wallet and start earning rewards today!"}</p>
                    <button class="cta-button large" onclick={on_get_started}>
                        {"🎮 Start Playing Now"}
                    </button>
                </div>
            </section>

            <style>
                {r#"
                /* Hero Section */
                .hero-section {
                    min-height: calc(100vh - 70px);
                    display: flex;
                    align-items: center;
                    padding: 80px 0;
                    background: linear-gradient(135deg, #000000 0%, #1a1a2e 50%, #16213e 100%);
                    position: relative;
                    overflow: hidden;
                }

                .hero-section::before {
                    content: '';
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background:
                        radial-gradient(circle at 20% 80%, rgba(0, 212, 255, 0.1) 0%, transparent 50%),
                        radial-gradient(circle at 80% 20%, rgba(139, 92, 246, 0.1) 0%, transparent 50%),
                        radial-gradient(circle at 40% 40%, rgba(16, 185, 129, 0.05) 0%, transparent 50%);
                    animation: cosmic-drift 20s ease-in-out infinite;
                }

                @keyframes cosmic-drift {
                    0%, 100% { transform: translate(0, 0) rotate(0deg); }
                    50% { transform: translate(-20px, -10px) rotate(1deg); }
                }

                .hero-container {
                    max-width: 1200px;
                    margin: 0 auto;
                    padding: 0 40px;
                    position: relative;
                    z-index: 1;
                }

                .hero-content {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 80px;
                    align-items: center;
                }

                .hero-title {
                    font-size: 4rem;
                    font-weight: 800;
                    line-height: 1.1;
                    margin-bottom: 24px;
                    color: white;
                }

                .highlight {
                    background: linear-gradient(45deg, #00d4ff, #0099cc);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    background-clip: text;
                }

                .hero-description {
                    font-size: 1.25rem;
                    line-height: 1.6;
                    color: #d0d0d0;
                    margin-bottom: 40px;
                }

                .hero-buttons {
                    display: flex;
                    gap: 20px;
                }

                .cta-button {
                    padding: 16px 32px;
                    font-size: 1.1rem;
                    font-weight: 600;
                    border: none;
                    border-radius: 12px;
                    cursor: pointer;
                    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
                    text-decoration: none;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    gap: 8px;
                }

                .cta-button.primary {
                    background: linear-gradient(45deg, #00d4ff, #0099cc);
                    color: white;
                }

                .cta-button.primary:hover {
                    transform: translateY(-2px);
                    box-shadow: 0 10px 25px rgba(0, 212, 255, 0.4);
                }

                .cta-button.secondary {
                    background: rgba(255, 255, 255, 0.1);
                    color: white;
                    border: 2px solid rgba(255, 255, 255, 0.2);
                }

                .cta-button.secondary:hover {
                    background: rgba(255, 255, 255, 0.15);
                    border-color: rgba(255, 255, 255, 0.3);
                    transform: translateY(-2px);
                }

                .cta-button.large {
                    padding: 20px 40px;
                    font-size: 1.3rem;
                    background: linear-gradient(45deg, #00d4ff, #0099cc);
                    color: white;
                }

                .cta-button.large:hover {
                    transform: translateY(-3px);
                    box-shadow: 0 15px 30px rgba(0, 212, 255, 0.4);
                }

                /* Game Preview */
                .hero-visual {
                    position: relative;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                }

                .game-preview {
                    width: 400px;
                    height: 300px;
                    position: relative;
                    background: rgba(0, 0, 0, 0.8);
                    border-radius: 16px;
                    border: 2px solid #333;
                    overflow: hidden;
                    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
                }

                .game-screen {
                    width: 100%;
                    height: 100%;
                    background: linear-gradient(90deg, #1a4f3a 0%, #2d5a3d 50%, #1a4f3a 100%);
                    position: relative;
                }

                .field {
                    width: 100%;
                    height: 100%;
                    position: relative;
                    background-image:
                        linear-gradient(90deg, transparent 49%, rgba(255,255,255,0.3) 50%, transparent 51%),
                        radial-gradient(circle at 50% 50%, transparent 30%, rgba(255,255,255,0.1) 31%, rgba(255,255,255,0.1) 32%, transparent 33%);
                }

                .player {
                    width: 20px;
                    height: 20px;
                    border-radius: 50%;
                    position: absolute;
                    animation: player-move 4s ease-in-out infinite;
                }

                .player-1 {
                    background: #00d4ff;
                    top: 40%;
                    left: 30%;
                    box-shadow: 0 0 15px rgba(0, 212, 255, 0.6);
                }

                .player-2 {
                    background: #ff6b6b;
                    top: 60%;
                    right: 30%;
                    animation-delay: -2s;
                    box-shadow: 0 0 15px rgba(255, 107, 107, 0.6);
                }

                .ball {
                    width: 12px;
                    height: 12px;
                    background: white;
                    border-radius: 50%;
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    animation: ball-bounce 2s ease-in-out infinite;
                    box-shadow: 0 0 10px rgba(255, 255, 255, 0.8);
                }

                .goal {
                    width: 8px;
                    height: 60px;
                    background: rgba(255, 255, 255, 0.8);
                    position: absolute;
                    top: 50%;
                    transform: translateY(-50%);
                }

                .goal-left {
                    left: 10px;
                }

                .goal-right {
                    right: 10px;
                }

                @keyframes player-move {
                    0%, 100% { transform: translate(0, 0); }
                    25% { transform: translate(20px, -10px); }
                    50% { transform: translate(10px, 15px); }
                    75% { transform: translate(-15px, 5px); }
                }

                @keyframes ball-bounce {
                    0%, 100% { transform: translate(-50%, -50%) scale(1); }
                    50% { transform: translate(-50%, -60%) scale(1.1); }
                }

                .floating-icons {
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    pointer-events: none;
                }

                .icon {
                    position: absolute;
                    font-size: 2rem;
                    animation: float-icon 6s ease-in-out infinite;
                }

                .icon-1 {
                    top: 10%;
                    right: 10%;
                    animation-delay: 0s;
                }

                .icon-2 {
                    top: 20%;
                    left: -10%;
                    animation-delay: -1.5s;
                }

                .icon-3 {
                    bottom: 20%;
                    right: -10%;
                    animation-delay: -3s;
                }

                .icon-4 {
                    bottom: 10%;
                    left: 10%;
                    animation-delay: -4.5s;
                }

                @keyframes float-icon {
                    0%, 100% { transform: translateY(0) rotate(0deg); opacity: 0.6; }
                    50% { transform: translateY(-20px) rotate(180deg); opacity: 1; }
                }

                /* Features Section */
                .features-section {
                    padding: 120px 0;
                    background: rgba(0, 0, 0, 0.8);
                }

                .features-container {
                    max-width: 1200px;
                    margin: 0 auto;
                    padding: 0 40px;
                }

                .section-title {
                    font-size: 3rem;
                    font-weight: 700;
                    text-align: center;
                    margin-bottom: 80px;
                    background: linear-gradient(45deg, #00d4ff, #0099cc);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    background-clip: text;
                }

                .features-grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
                    gap: 40px;
                }

                .feature-card {
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    border-radius: 16px;
                    padding: 40px 32px;
                    text-align: center;
                    transition: all 0.3s ease;
                }

                .feature-card:hover {
                    transform: translateY(-5px);
                    border-color: rgba(0, 212, 255, 0.3);
                    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
                }

                .feature-icon {
                    font-size: 3rem;
                    margin-bottom: 24px;
                }

                .feature-card h3 {
                    font-size: 1.5rem;
                    font-weight: 600;
                    margin-bottom: 16px;
                    color: white;
                }

                .feature-card p {
                    color: #d0d0d0;
                    line-height: 1.6;
                }

                /* Stats Section */
                .stats-section {
                    padding: 80px 0;
                    background: linear-gradient(45deg, rgba(0, 212, 255, 0.05), rgba(139, 92, 246, 0.05));
                }

                .stats-container {
                    max-width: 1000px;
                    margin: 0 auto;
                    padding: 0 40px;
                }

                .stats-grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 40px;
                }

                .stat-item {
                    text-align: center;
                }

                .stat-number {
                    font-size: 3rem;
                    font-weight: 800;
                    color: #00d4ff;
                    margin-bottom: 8px;
                }

                .stat-label {
                    color: #d0d0d0;
                    font-size: 1.1rem;
                    font-weight: 500;
                }

                /* CTA Section */
                .cta-section {
                    padding: 120px 0;
                    background: rgba(0, 0, 0, 0.9);
                    text-align: center;
                }

                .cta-container {
                    max-width: 800px;
                    margin: 0 auto;
                    padding: 0 40px;
                }

                .cta-section h2 {
                    font-size: 2.5rem;
                    font-weight: 700;
                    margin-bottom: 24px;
                    color: white;
                }

                .cta-section p {
                    font-size: 1.2rem;
                    color: #d0d0d0;
                    margin-bottom: 40px;
                }

                /* Login Modal */
                .login-modal {
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: rgba(0, 0, 0, 0.8);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    z-index: 2000;
                    padding: 20px;
                }

                .login-modal-content {
                    position: relative;
                    max-width: 500px;
                    width: 100%;
                    max-height: 90vh;
                    overflow-y: auto;
                    background: transparent;
                }

                .close-btn {
                    position: absolute;
                    top: -40px;
                    right: 0;
                    background: rgba(255, 255, 255, 0.1);
                    border: none;
                    color: white;
                    font-size: 24px;
                    width: 40px;
                    height: 40px;
                    border-radius: 50%;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    transition: all 0.3s ease;
                    z-index: 2001;
                }

                .close-btn:hover {
                    background: rgba(255, 255, 255, 0.2);
                    transform: rotate(90deg);
                }

                /* Mobile Responsive */
                @media (max-width: 1024px) {
                    .hero-content {
                        grid-template-columns: 1fr;
                        gap: 60px;
                        text-align: center;
                    }

                    .hero-title {
                        font-size: 3rem;
                    }

                    .game-preview {
                        width: 350px;
                        height: 250px;
                    }
                }

                @media (max-width: 768px) {
                    .hero-container, .features-container, .stats-container, .cta-container {
                        padding: 0 20px;
                    }

                    .hero-title {
                        font-size: 2.5rem;
                    }

                    .hero-description {
                        font-size: 1.1rem;
                    }

                    .hero-buttons {
                        flex-direction: column;
                        gap: 16px;
                    }

                    .section-title {
                        font-size: 2.2rem;
                    }

                    .game-preview {
                        width: 300px;
                        height: 200px;
                    }

                    .features-grid {
                        grid-template-columns: 1fr;
                        gap: 30px;
                    }

                    .stats-grid {
                        grid-template-columns: repeat(2, 1fr);
                        gap: 30px;
                    }

                    .stat-number {
                        font-size: 2.2rem;
                    }
                }

                @media (max-width: 480px) {
                    .hero-title {
                        font-size: 2rem;
                    }

                    .cta-button {
                        padding: 14px 28px;
                        font-size: 1rem;
                    }

                    .game-preview {
                        width: 280px;
                        height: 180px;
                    }

                    .stats-grid {
                        grid-template-columns: 1fr;
                        gap: 20px;
                    }
                }
                "#}
            </style>
        </>
    }
}