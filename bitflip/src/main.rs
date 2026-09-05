#![allow(clippy::needless_return)]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> bitflip::AppResult<()> {
	use std::sync::Arc;

	use axum::Router;
	use axum::extract::State;
	use axum::http::StatusCode;
	use axum::routing::get;
	use bitflip::app::*;
	use bitflip::db::Db;
	use bitflip::image_generator::section_image_handler;
	use bitflip::state::AppState;
	use bitflip::state::AppStateConfig;
	use leptos::prelude::*;
	use leptos_axum::LeptosRoutes;
	use leptos_axum::generate_route_list;
	use tower_http::compression::CompressionLayer;
	use tower_http::compression::CompressionLevel;
	use tower_http::compression::Predicate;
	use tower_http::compression::predicate::NotForContentType;
	use tower_http::compression::predicate::SizeAbove;

	simple_logger::init_with_level(log::Level::Info)?;
	let conf = get_configuration(None)?;
	let addr = conf.leptos_options.site_addr;
	let leptos_options = conf.leptos_options;
	let config = Arc::new(AppStateConfig::from_env()?);
	let db = Db::try_new(config.database_url.as_str()).await?;
	sqlx::migrate!("../migrations")
		.run(db.as_sqlx_pool())
		.await?;
	let state = AppState::builder()
		.leptos(leptos_options.clone())
		.config(config)
		.db(db)
		.build();
	// Generate the list of routes in your Leptos App
	let routes = generate_route_list(App);

	let predicate = SizeAbove::new(1500) // files smaller than 1501 bytes are not compressed, since the MTU (Maximum Transmission
		// Unit) of a TCP packet is 1500 bytes
		.and(NotForContentType::GRPC)
		.and(NotForContentType::IMAGES)
		// prevent compressing assets that are already statically compressed
		.and(NotForContentType::const_new("application/javascript"))
		.and(NotForContentType::const_new("application/wasm"))
		.and(NotForContentType::const_new("text/css"));

	let app = Router::new()
		.leptos_routes(&state, routes, {
			let leptos_options = leptos_options.clone();
			move || shell(leptos_options.clone())
		})
		.layer(
			CompressionLayer::new()
				.quality(CompressionLevel::Fastest)
				.compress_when(predicate),
		)
		.route(
			"/game/{game_index}/section-image/{section_index}",
			get(section_image_handler),
		)
		.route(
			"/healthz",
			get(|State(state): State<AppState>| async move {
				sqlx::query_scalar::<_, i64>("SELECT 1")
					.fetch_one(state.db.as_sqlx_pool())
					.await
					.map(|_| StatusCode::NO_CONTENT)
					.map_err(bitflip::AppError::from)
			}),
		)
		.fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
		.with_state(state);

	// run our app with hyper
	// `axum::Server` is a re-export of `hyper::Server`
	log::info!("listening on http://{}", &addr);
	let listener = tokio::net::TcpListener::bind(&addr).await?;

	axum::serve(listener, app.into_make_service()).await?;

	Ok(())
}

// client-only stuff for Trunk
#[cfg(not(feature = "ssr"))]
pub fn main() {
	use bitflip::app::*;
	use leptos::prelude::*;

	console_error_panic_hook::set_once();
	leptos::mount::mount_to_body(App);
}
