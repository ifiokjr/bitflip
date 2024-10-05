#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
	use axum::Router;
	use axum::routing::get;
	use bitflip::app::*;
	use bitflip::image_generator::section_image_handler;
	use leptos::prelude::*;
	use leptos_axum::LeptosRoutes;
	use leptos_axum::generate_route_list;
	use tower_http::compression::CompressionLayer;
	use tower_http::compression::CompressionLevel;
	use tower_http::compression::Predicate;
	use tower_http::compression::predicate::NotForContentType;
	use tower_http::compression::predicate::SizeAbove;

	simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");
	let conf = get_configuration(None).unwrap();
	let addr = conf.leptos_options.site_addr;
	let leptos_options = conf.leptos_options;
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
		.leptos_routes(&leptos_options, routes, {
			let leptos_options = leptos_options.clone();
			move || shell(leptos_options.clone())
		})
		.layer(
			CompressionLayer::new()
				.quality(CompressionLevel::Fastest)
				.compress_when(predicate),
		)
		.route("/section-image/:section_index", get(section_image_handler))
		.fallback(leptos_axum::file_and_error_handler(shell))
		.with_state(leptos_options);

	// run our app with hyper
	// `axum::Server` is a re-export of `hyper::Server`
	log!("listening on http://{}", &addr);
	let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
	axum::serve(listener, app.into_make_service())
		.await
		.unwrap();
}

// client-only stuff for Trunk
#[cfg(not(feature = "ssr"))]
pub fn main() {
	use bitflip::app::*;
	use leptos::prelude::*;

	console_error_panic_hook::set_once();
	leptos::mount::mount_to_body(App);
}
